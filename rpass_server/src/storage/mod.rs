mod storage;
mod record;

#[mockall_double::double]
pub use storage::Storage;
pub use storage::*;
pub use record::*;
pub use rpass::key::*;
