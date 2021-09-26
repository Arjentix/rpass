mod storage;
mod record;

#[mockall_double::double]
pub use storage::Storage;
pub use record::Record;
pub use rpass::key::Key;
