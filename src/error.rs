use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IntConvError,
    InvalidMagicNumber,
    InvalidDpSize,
    InvalidDpHash,
    InvalidDpCount,
    InvalidTimeStep,
    InvalidStreamLen,
    UpdateTooEarly,
    UpdateTooLate,
    InvalidSkip,
    MaxSkipExceeded, 
    OutOfRangePast,
    OutOfRangeFuture,
    IoError(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            IntConvError => write!(f, "integer conversion overflowed"),
            InvalidMagicNumber => write!(f, "invalid magic number"),
            InvalidDpSize => write!(f, "dp size must be non-zero"),
            InvalidDpHash => write!(f, "invalid datapoint hash value"),
            InvalidDpCount => write!(f, "dp count must be at least 2"),
            InvalidTimeStep => write!(f, "time step must be non-zero"),
            InvalidStreamLen => write!(f, "i/o stream has wrong length"),
            UpdateTooEarly => write!(f, "update time is too early"),
            UpdateTooLate => write!(f, "update time is too late"),
            InvalidSkip => write!(f, "max fwd skip cannot be greater than dp count - 2"),
            MaxSkipExceeded => write!(f, "max fwd skip value exceeded"), 
            OutOfRangePast => write!(f, "requested time is too far in the past"),
            OutOfRangeFuture => write!(f, "requested time is in the future"),
            IoError(e) => e.fmt(f),
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        use Error::*;
        matches!(
            (self, other),
            (IntConvError, IntConvError)
                | (InvalidMagicNumber, InvalidMagicNumber)
                | (InvalidDpSize, InvalidDpSize)
                | (InvalidDpHash, InvalidDpHash)
                | (InvalidDpCount, InvalidDpCount)
                | (InvalidTimeStep, InvalidTimeStep)
                | (InvalidStreamLen, InvalidStreamLen)
                | (UpdateTooEarly, UpdateTooEarly)
                | (UpdateTooLate, UpdateTooLate)
                | (MaxSkipExceeded, MaxSkipExceeded)
                | (OutOfRangePast, OutOfRangePast)
                | (OutOfRangeFuture, OutOfRangeFuture)
                | (IoError(_), IoError(_))
        )
    }
}
