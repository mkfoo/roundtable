use super::prelude::*;
use std::io::SeekFrom;

const RTDB: u32 = 0x52544442;

super::datapoint! {
    pub struct Header {
        magic: u32,
        dp_size: u64,
        dp_hash: u64,
        dp_count: u64,
        t_start: u64,
        t_step: u64,
        t_updated: u64,
    }
}

impl Header {
    pub fn new<T: DataPoint>(dp: &T, dp_count: u64, t_start: u64, t_step: u64) -> Self {
        Self {
            magic: RTDB,
            dp_size: dp.get_size(),
            dp_hash: dp.get_hash(),
            dp_count,
            t_start,
            t_step,
            t_updated: t_start,
        }
    }

    pub fn validate<T: DataPoint>(&self, dp: &T) -> Result<()> {
        use Error::*;

        if self.magic != RTDB {
            return Err(InvalidMagicNumber);
        }

        if self.dp_size != dp.get_size() {
            return Err(InvalidDpSize);
        }

        if self.dp_hash != dp.get_hash() {
            return Err(InvalidDpHash);
        }

        if self.dp_count == 0 {
            return Err(InvalidDpCount);
        }

        if self.t_step == 0 {
            return Err(InvalidTimeStep);
        }

        Ok(())
    }

    fn get_slot(&self, t_now: u64) -> u64 {
        let elapsed = t_now - self.t_start;
        let t_len = self.t_step * self.dp_count;
        elapsed % t_len / self.t_step
    }

    fn get_offset(&self, slot: u64) -> u64 {
        slot * self.dp_size + self.get_size()
    }

    fn get_earliest(&self) -> u64 {
        let t1 = self
            .t_updated
            .saturating_sub(self.t_step * (self.dp_count - 1));

        if t1 > self.t_start {
            t1
        } else {
            self.t_start
        }
    }

    fn check_access_time(&self, t: u64) -> Result<()> {
        if t > self.t_updated {
            return Err(Error::OutOfRangeFuture);
        }

        if t < self.get_earliest() {
            return Err(Error::OutOfRangePast);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Table<T, U>
where
    T: DataPoint,
    U: Read + Write + Seek + Sized,
{
    max_skip: u64,
    skip_mode: FwdSkipMode,
    header: Header,
    data: U,
    buf: T,
    slot: u64,
}

impl<T, U> Table<T, U>
where
    T: DataPoint + Copy + Default,
    U: Read + Write + Seek + Sized,
{
    pub fn new(
        opts: Options,
        dp_count: u64,
        t_start: u64,
        t_step: u64,
        dp: &T,
        mut data: U,
    ) -> Result<Self> {
        let header = Header::new(dp, dp_count, t_start, t_step);
        header.validate(dp)?;
        header.write_out(&mut data).map_err(Error::IoError)?;
        dp.write_out(&mut data).map_err(Error::IoError)?;

        Ok(Self {
            max_skip: opts.max_fwd_skip,
            skip_mode: opts.fwd_skip_mode,
            header,
            data,
            buf: T::default(),
            slot: 0,
        })
    }

    pub fn insert(&mut self, t_now: u64, dp: &T) -> Result<()> {
        let upd = self.header.t_updated / self.header.t_step;
        let now = t_now / self.header.t_step;

        match now.saturating_sub(upd) {
            0 => return Err(Error::UpdateTooEarly),
            1 => {
                self.seek_to(self.header.t_updated)?;
                self.seek_fwd()?;
            }
            n if n < self.header.dp_count => self.skip_fwd(n - 1, dp)?,
            _ => return Err(Error::UpdateTooLate),
        }

        self.write_out(dp)?;
        self.update_header(t_now)
    }

    pub fn get(&mut self, t: u64) -> Result<&T> {
        self.header.check_access_time(t)?;
        self.seek_to(t)?;
        self.read_in()?;
        Ok(&self.buf)
    }

    fn skip_fwd(&mut self, skip: u64, dp: &T) -> Result<()> {
        use FwdSkipMode::*;

        if skip > self.max_skip {
            return Err(Error::UpdateTooLate);
        }

        self.seek_to(self.header.t_updated)?;
        self.read_in()?;

        for i in 0..skip {
            match self.skip_mode {
                DoNothing => self.seek_fwd()?,
                Linear => self.skip_linear(i, skip, dp)?,
                Nearest => self.skip_nearest(i, skip, dp)?,
                Zeroed => self.skip_default()?,
            }
        }

        Ok(())
    }

    fn skip_linear(&mut self, _i: u64, _skip: u64, _dp: &T) -> Result<()> {
        todo!();
    }

    fn skip_nearest(&mut self, i: u64, skip: u64, dp: &T) -> Result<()> {
        if i < skip / 2 {
            self.write_out_buf()
        } else {
            self.write_out(dp)
        }
    }

    fn skip_default(&mut self) -> Result<()> {
        self.buf = T::default();
        self.write_out_buf()
    }

    fn increment_slot(&mut self) -> Result<()> {
        self.slot = (self.slot + 1) % self.header.dp_count;

        if self.slot == 0 {
            self.seek_from_start(self.header.get_size())?;
        }

        Ok(())
    }

    fn seek_from_start(&mut self, offset: u64) -> Result<u64> {
        self.data
            .seek(SeekFrom::Start(offset))
            .map_err(Error::IoError)
    }

    fn seek_to(&mut self, t: u64) -> Result<u64> {
        self.slot = self.header.get_slot(t);
        let offset = self.header.get_offset(self.slot);
        self.seek_from_start(offset)
    }

    fn seek_fwd(&mut self) -> Result<()> {
        let ioff = i64::try_from(self.header.dp_size).map_err(|_| Error::IntConvError)?;
        self.data
            .seek(SeekFrom::Current(ioff))
            .map_err(Error::IoError)?;
        self.increment_slot()
    }

    fn write_out<D: DataPoint>(&mut self, dp: &D) -> Result<()> {
        dp.write_out(&mut self.data).map_err(Error::IoError)?;
        self.increment_slot()
    }

    fn write_out_buf(&mut self) -> Result<()> {
        self.buf.write_out(&mut self.data).map_err(Error::IoError)?;
        self.increment_slot()
    }

    fn read_in(&mut self) -> Result<()> {
        self.buf.read_in(&mut self.data).map_err(Error::IoError)?;
        self.increment_slot()
    }

    fn update_header(&mut self, t_now: u64) -> Result<()> {
        let offset = self.header.get_size() - self.header.t_updated.get_size();
        self.seek_from_start(offset)?;
        t_now.write_out(&mut self.data).map_err(Error::IoError)?;
        self.header.t_updated = t_now;
        Ok(())
    }
}
