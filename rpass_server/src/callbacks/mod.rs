pub mod register;
pub mod login;
pub mod confirm_login;
pub mod delete_me;
pub mod new_record;

#[mockall_double::double]
pub use crate::storage::Storage;
pub use register::register;
pub use login::login;
pub use confirm_login::confirm_login;
pub use delete_me::delete_me;
pub use new_record::new_record;

use std::sync::{Arc, RwLock};

use crate::request_dispatcher::{ArgIter};
use crate::session::Session;

type AsyncStorage = Arc<RwLock<Storage>>;
