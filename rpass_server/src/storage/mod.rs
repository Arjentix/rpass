mod storage;
mod record;

#[mockall_double::double]
pub use storage::Storage;
pub use storage::Error;
pub use record::Record;
pub use rpass::key::Key;
