pub mod error;
pub mod register;
pub mod login;
pub mod confirm_login;
pub mod delete_me;
pub mod quit;
pub mod new_record;
pub mod show_record;

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
pub type Result<T> = std::result::Result<T, Error>;

use std::sync::{Arc, RwLock};

use crate::request_dispatcher::{ArgIter};
use crate::session::Session;

type AsyncStorage = Arc<RwLock<storage::Storage>>;
