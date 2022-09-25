use super::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::Cursor;
use std::path::Path;

pub fn in_memory<T: DataPoint + Copy + Default>(
    opts: Options,
    first_dp: T,
) -> Result<Table<T, Cursor<Vec<u8>>>> {
    let data = if opts.preallocate {
        let len = usize::try_from(first_dp.get_size() * opts.dp_count())
            .map_err(|_| Error::IntConvError)?;
        Cursor::new(vec![0; len])
    } else {
        Cursor::new(vec![])
    };

    Table::new(&opts, &first_dp, data)
}

pub fn in_file<T: DataPoint + Copy + Default, P: AsRef<Path>>(
    opts: Options,
    first_dp: T,
    path: P,
) -> Result<Table<T, File>> {
    let file = if opts.overwrite {
        OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .open(path)
    } else {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
    }
    .map_err(Error::IoError)?;

    if opts.preallocate {
        let len = first_dp.get_size() * opts.dp_count();
        file.set_len(len).map_err(Error::IoError)?;
    }

    Table::new(&opts, &first_dp, file)
}
