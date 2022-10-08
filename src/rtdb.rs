use super::error::Error;
use super::prelude::*;
use super::Result;
use std::io::SeekFrom;

const RTDB: u32 = 0x42445452;

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
    pub fn new<T: DataPoint>(opts: &Options, dp: &T) -> Self {
        Self {
            magic: RTDB,
            dp_size: dp.get_size(),
            dp_hash: dp.get_hash(),
            dp_count: opts.dp_count(),
            t_start: opts.t_start,
            t_step: opts.t_step,
            t_updated: opts.t_start,
        }
    }

    pub fn validate<T: DataPoint>(&self, opts: &Options, dp: &T) -> Result<()> {
        use Error::*;

        if self.magic != RTDB {
            return Err(InvalidMagicNumber);
        }

        if self.dp_size != dp.get_size() {
            return Err(InvalidDpSize);
        }

        if !opts.ignore_hash && self.dp_hash != dp.get_hash() {
            return Err(InvalidDpHash);
        }

        if self.t_step == 0 {
            return Err(InvalidTimeStep);
        }

        if self.dp_count < 2 {
            return Err(InvalidDpCount);
        }

        if opts.max_fwd_skip > self.dp_count - 2 {
            return Err(Error::InvalidSkip);
        }

        Ok(())
    }

    fn check_stream_len<R: Read + Seek>(&self, stream: &mut R) -> Result<()> {
        let len = stream.seek(SeekFrom::End(0)).map_err(Error::IoError)?;
        stream.seek(SeekFrom::Start(0)).map_err(Error::IoError)?;

        if self.get_first() > self.t_start {
            return self.check_full_len(len);
        }

        self.check_partial_len(len)
    }

    fn check_full_len(&self, len: u64) -> Result<()> {
        if len != self.dp_count * self.dp_size + self.get_size() {
            println!("full len {} {}", len, self.dp_count * self.dp_size);
            return Err(Error::InvalidStreamLen);
        }

        Ok(())
    }

    fn check_partial_len(&self, len: u64) -> Result<()> {
        let dps = self.get_slot(self.t_updated) + 1;

        if len < dps * self.dp_size + self.get_size() {
            return Err(Error::InvalidStreamLen);
        }

        if len > self.dp_count * self.dp_size + self.get_size() {
            return Err(Error::InvalidStreamLen);
        }

        Ok(())
    }

    fn round_down(&self, t: u64) -> u64 {
        let d = t - self.t_start;
        self.t_start + d - d % self.t_step
    }

    fn get_slot(&self, t_now: u64) -> u64 {
        let elapsed = t_now - self.t_start;
        let t_total = self.t_step * self.dp_count;
        elapsed % t_total / self.t_step
    }

    fn get_offset(&self, slot: u64) -> u64 {
        slot * self.dp_size + self.get_size()
    }

    fn get_first(&self) -> u64 {
        let upd = self.round_down(self.t_updated);
        let elapsed = upd - self.t_start;
        let t_total = self.t_step * self.dp_count;

        if elapsed < t_total {
            return self.t_start;
        }

        upd - (t_total - self.t_step)
    }

    fn get_delta(&self, s: u64, e: u64) -> u64 {
        let start = self.round_down(s);
        let end = self.round_down(e);
        end / self.t_step - start / self.t_step
    }

    fn check_access_time(&self, t: u64) -> Result<()> {
        if t > self.t_updated {
            return Err(Error::OutOfRangeFuture);
        }

        if t < self.get_first() {
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
    pub fn new(opts: &Options, dp: &T, mut data: U) -> Result<Self> {
        let header = Header::new(opts, dp);
        header.validate(opts, dp)?;
        data.seek(SeekFrom::Start(0)).map_err(Error::IoError)?;
        header.write_out(&mut data).map_err(Error::IoError)?;
        dp.write_out(&mut data).map_err(Error::IoError)?;
        header.check_stream_len(&mut data)?;

        Ok(Self {
            max_skip: opts.max_fwd_skip,
            skip_mode: opts.fwd_skip_mode,
            header,
            data,
            buf: T::default(),
            slot: 0,
        })
    }

    pub fn load(opts: &Options, dp: &T, mut data: U) -> Result<Self> {
        let mut header = Header::default();
        data.seek(SeekFrom::Start(0)).map_err(Error::IoError)?;
        header.read_in(&mut data).map_err(Error::IoError)?;
        header.validate(opts, dp)?;
        header.check_stream_len(&mut data)?;

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
        if t_now <= self.header.t_updated {
            return Err(Error::UpdateTooEarly);
        }

        let delta = self.header.get_delta(self.header.t_updated, t_now);

        match delta {
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

    pub fn first(&mut self) -> Result<(u64, &T)> {
        let t = self.header.get_first();
        self.get(t).map(|v| (t, v))
    }

    pub fn last(&mut self) -> Result<(u64, &T)> {
        let t = self.header.t_updated;
        self.get(t).map(|v| (t, v))
    }

    pub fn iter(&mut self) -> Result<Iter<T, U>> {
        let now = self.header.get_first();
        let end = self.header.round_down(self.header.t_updated);
        self.seek_to(now)?;

        Ok(Iter {
            table: self,
            now,
            end,
        })
    }

    pub fn range(&mut self, start: u64, end: u64) -> Result<Iter<T, U>> {
        self.header.check_access_time(start)?;
        self.header.check_access_time(end)?;
        let now = self.header.round_down(start);
        let end = self.header.round_down(end);
        self.seek_to(now)?;

        Ok(Iter {
            table: self,
            now,
            end,
        })
    }

    pub fn into_inner(self) -> U {
        self.data
    }

    fn skip_fwd(&mut self, skip: u64, dp: &T) -> Result<()> {
        use FwdSkipMode::*;

        if skip > self.max_skip {
            return Err(Error::MaxSkipExceeded);
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

    fn increment(&mut self) -> Result<()> {
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
        self.increment()
    }

    fn write_out<D: DataPoint>(&mut self, dp: &D) -> Result<()> {
        dp.write_out(&mut self.data).map_err(Error::IoError)?;
        self.increment()
    }

    fn write_out_buf(&mut self) -> Result<()> {
        self.buf.write_out(&mut self.data).map_err(Error::IoError)?;
        self.increment()
    }

    fn read_in(&mut self) -> Result<()> {
        self.buf.read_in(&mut self.data).map_err(Error::IoError)?;
        self.increment()
    }

    fn update_header(&mut self, t_now: u64) -> Result<()> {
        let offset = self.header.get_size() - self.header.t_updated.get_size();
        self.seek_from_start(offset)?;
        t_now.write_out(&mut self.data).map_err(Error::IoError)?;
        self.header.t_updated = t_now;
        Ok(())
    }
}

pub struct Iter<'a, T, U>
where
    T: DataPoint + Copy + Default,
    U: Read + Write + Seek + Sized,
{
    table: &'a mut Table<T, U>,
    now: u64,
    end: u64,
}

impl<'a, T, U> Iterator for Iter<'a, T, U>
where
    T: DataPoint + Copy + Default,
    U: Read + Write + Seek + Sized,
{
    type Item = (u64, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.now <= self.end {
            let t = self.now;
            self.now += self.table.header.t_step;
            self.table.read_in().ok()?;
            Some((t, self.table.buf))
        } else {
            None
        }
    }
}
