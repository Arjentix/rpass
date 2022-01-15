mod confirm_login;
mod delete_me;
mod delete_record;
mod error;
mod list_records;
mod login;
mod new_record;
mod register;
mod show_record;

mod utils;

pub use crate::storage;
pub use confirm_login::confirm_login;
pub use delete_me::delete_me;
pub use delete_record::delete_record;
pub use error::Error;
pub use list_records::list_records;
pub use login::login;
pub use new_record::new_record;
pub use register::register;
pub use show_record::show_record;
pub type Result<T> = std::result::Result<T, Error>;

use crate::request_dispatcher::ArgIter;
use crate::session;

use crate::AsyncStorage;

#[cfg(test)]
use std::sync::{Arc, RwLock};
#[cfg(test)]
type AsyncUserStorage = Arc<RwLock<storage::UserStorage>>;
