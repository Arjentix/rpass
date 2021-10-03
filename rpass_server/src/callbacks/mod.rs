pub mod register;
pub mod login;
pub mod confirm_login;
pub mod delete_me;
pub mod new_record;
pub mod quit;

pub use crate::storage;
pub use register::register;
pub use login::login;
pub use confirm_login::confirm_login;
pub use delete_me::delete_me;
pub use new_record::new_record;
pub use quit::quit;

use std::sync::{Arc, RwLock};

use crate::request_dispatcher::{ArgIter};
use crate::session::Session;

type AsyncStorage = Arc<RwLock<storage::Storage>>;
