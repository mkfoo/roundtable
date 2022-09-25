use super::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::Cursor;
use std::path::Path;

pub fn from_buffer<T: DataPoint + Copy + Default, U: AsRef<[u8]>>(
    opts: Options,
    buf: U,
) -> Result<Table<T, Cursor<U>>>
where
    Cursor<U>: Read + Write + Seek + Sized,
{
    let dp = T::default();
    let data = Cursor::new(buf);
    Table::load(&opts, &dp, data)
}

pub fn from_file<T: DataPoint + Copy + Default, P: AsRef<Path>>(
    opts: Options,
    path: P,
) -> Result<Table<T, File>> {
    let dp = T::default();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(Error::IoError)?;
    Table::load(&opts, &dp, file)
}
