pub mod error;
pub mod key;
pub mod record;
pub mod session;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
