pub mod create;
pub mod data;
pub mod error;
pub mod load;
pub mod options;
pub mod rtdb;

pub type Error = self::error::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub mod prelude {
    pub use super::data::DataPoint;
    pub use super::options::{FwdSkipMode, Options};
    pub use super::rtdb::Table;
    pub type InMemoryTable<T> = Table<T, std::io::Cursor<Vec<u8>>>;
}
