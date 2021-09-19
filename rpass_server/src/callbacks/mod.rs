pub mod errors;
pub mod register;
pub mod login;
pub mod delete_me;

use mockall_double::double;
#[double]
pub use crate::storage::Storage;
pub use register::register;
pub use login::login;
pub use delete_me::delete_me;

use std::sync::{Arc, RwLock};

use crate::request_dispatcher::{ArgIter};
use crate::session::Session;
use errors::*;

type AsyncStorage = Arc<RwLock<Storage>>;

/// Second and final part of user logging. Reads encrypted confirmation string
/// from `arg_iter`, decrypts it with `storage.sec_key` and checks if it is equal to the
/// `session.login_confirmation`.
/// 
/// Sets `session.is_authorized` to *true* and returns *Ok("Ok")* if everything
/// is good
/// 
/// See [`login()`] function for first part
/// 
/// # Errors
/// 
/// * `UnacceptableRequestAtThisState` - if there isn't *login_confirmation* in `session` or
/// user already authorized
/// * `EmptyConfirmationString` - if confirmation string wasn't provided
pub fn confirm_login(storage: AsyncStorage, session: &mut Session, arg_iter: ArgIter)
        -> Result<String, ConfirmLoginError> {
    if session.login_confirmation.is_none() || session.is_authorized {
        return Err(ConfirmLoginError::UnacceptableRequestAtThisState);
    }

    let encrypted_confirmation = arg_iter.next()
        .ok_or(ConfirmLoginError::EmptyConfirmationString)?;

    let sec_key;
    {
        let storage_read = storage.read().unwrap();
        sec_key = storage_read.get_sec_key().clone();
    }

    let confirmation = sec_key.decrypt(encrypted_confirmation);
    if &confirmation != session.login_confirmation.as_ref().unwrap() {
        return Err(ConfirmLoginError::EmptyConfirmationString);
    }
    
    session.login_confirmation = None;
    session.is_authorized = true;
    Ok("Ok".to_owned())
}
