use super::{AsyncStorage, Session, ArgIter};

/// Second and final part of user logging. Reads encrypted confirmation string
/// from `arg_iter`, decrypts it with `storage.sec_key` and checks if it is equal to the
/// `session.login_confirmation`.
/// 
/// Sets `session.is_authorized` to *true* and returns *Ok("Ok")* if everything
/// is good
/// 
/// See [`super::login()`] function for first part
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

#[derive(thiserror::Error, Debug)]
pub enum ConfirmLoginError {
    #[error("unacceptable request at this state")]
    UnacceptableRequestAtThisState,

    #[error("empty confirmation string")]
    EmptyConfirmationString
}
