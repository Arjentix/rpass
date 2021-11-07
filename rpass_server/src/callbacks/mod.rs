mod error;
mod register;
mod login;
mod confirm_login;
mod delete_me;
mod quit;
mod new_record;
mod show_record;
mod list_records;
mod delete_record;

mod utils;

pub use crate::storage;
pub use error::Error;
pub use register::register;
pub use login::login;
pub use confirm_login::confirm_login;
pub use delete_me::delete_me;
pub use quit::quit;
pub use new_record::new_record;
pub use show_record::show_record;
pub use list_records::list_records;
pub use delete_record::delete_record;
pub type Result<T> = std::result::Result<T, Error>;

use crate::request_dispatcher::ArgIter;
use crate::session;

use crate::AsyncStorage;

#[cfg(test)]
use std::sync::{Arc, RwLock};
#[cfg(test)]
type AsyncUserStorage = Arc<RwLock<storage::UserStorage>>;
