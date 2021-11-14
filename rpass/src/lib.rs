pub mod key;
pub mod session;
mod error;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
