pub mod errors;
pub mod delete_me;
pub mod register;

use mockall_double::double;
#[double]
pub use crate::storage::Storage;
pub use delete_me::delete_me;
pub use register::register;

use std::sync::{Arc, RwLock};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use crate::request_dispatcher::{ArgIter};
use crate::session::Session;
use errors::*;

type AsyncStorage = Arc<RwLock<Storage>>;

/// First part of user logging. Reads username from `arg_iter`, gets his key
/// from `storage` and writes random encrypted string into
/// `session.login_confirmation`. Returns *Ok() with login confirmation* in success
/// 
/// The next step user should decrypt that random confirmation string,
/// encrypt if with storage public key and send it back.
/// 
/// See [`confirm_login()`] function for second part
/// 
/// # Errors
/// 
/// * `EmptyUsername` - if no username was provided
/// * `NoSuchUser` - if user with such username doesn't exist
pub fn login(storage: AsyncStorage, session: &mut Session, arg_iter: ArgIter)
        -> Result<String, LoginError> {
    let username = arg_iter.next().ok_or(LoginError::EmptyUsername)?;

    let user_pub_key = {
        let storage_read = storage.read().unwrap();
        storage_read.get_user_pub_key(username)?
    };

    const RAND_STRING_LENGTH: usize = 30;
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(RAND_STRING_LENGTH)
        .map(char::from)
        .collect();

    session.login_confirmation = Some(user_pub_key.encrypt(&rand_string));
    session.is_authorized = false;
    session.username = username.to_owned();
    Ok(session.login_confirmation.as_ref().unwrap().clone())
}

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
