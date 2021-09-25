use super::{AsyncStorage, Session, ArgIter};

/// Second and final part of user logging. Reads encrypted confirmation string
/// from `arg_iter`, decrypts it with `storage.sec_key` and checks if it is
/// equal to the `session.login_confirmation`.
/// 
/// Sets `session.is_authorized` to *true* and returns *Ok("Ok")* if everything
/// is good
/// 
/// See [`super::login()`] function for first part
/// 
/// # Errors
/// 
/// * `UnacceptableRequestAtThisState` - if there isn't *login_confirmation* in
/// `session` or user already authorized
/// * `EmptyConfirmationString` - if confirmation string wasn't provided
/// * `InvalidConfirmationString` - if confirmation string isn't equal to the
/// one stored in `session.login_confirmation`
pub fn confirm_login(storage: AsyncStorage, session: &mut Session,
        arg_iter: ArgIter) -> Result<String, ConfirmLoginError> {
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

    let confirmation = sec_key.decrypt(&encrypted_confirmation);
    if &confirmation != session.login_confirmation.as_ref().unwrap() {
        return Err(ConfirmLoginError::InvalidConfirmationString);
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
    EmptyConfirmationString,

    #[error("invalid confirmation string")]
    InvalidConfirmationString
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Key;

    #[test]
    fn test_ok() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            login_confirmation: Some(String::from("confirmation")),
            is_authorized: false,
            username: String::default()
        };
        let (pub_key, sec_key) = Key::generate_pair();
        let encrypted_confirmation = pub_key.encrypt(
            session.login_confirmation.as_ref().unwrap());
        let mut arg_iter = encrypted_confirmation.split_whitespace();

        mock_storage.write().unwrap().expect_get_sec_key().times(1)
            .return_const(sec_key);
        let res = confirm_login(mock_storage, &mut session, &mut arg_iter);
        assert_eq!(res.unwrap(), "Ok");
        assert!(session.login_confirmation.is_none());
        assert!(session.is_authorized);
    }

    #[test]
    fn test_unacceptable_request_at_this_state() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            login_confirmation : None,
            is_authorized : false,
            username : String::default()
        };

        let mut arg_iter = "".split_whitespace();

        let res = confirm_login(mock_storage.clone(), &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(ConfirmLoginError::UnacceptableRequestAtThisState)));

        session.login_confirmation = Some(String::default());
        session.is_authorized = true;
        let res = confirm_login(mock_storage, &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(ConfirmLoginError::UnacceptableRequestAtThisState)));
    }

    #[test]
    fn test_empty_confirmation_string() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            login_confirmation: Some(String::default()),
            is_authorized: false,
            username: String::default()
        };
        let mut arg_iter = "".split_whitespace();

        let res = confirm_login(mock_storage, &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(ConfirmLoginError::EmptyConfirmationString)));
    }

    #[test]
    fn test_invalid_confirmation_string() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            login_confirmation: Some(String::from("confirmation")),
            is_authorized: false,
            username: String::default()
        };
        let (pub_key, sec_key) = Key::generate_pair();
        let encrypted_confirmation = pub_key.encrypt("wrong_confirmation");
        let mut arg_iter = encrypted_confirmation.split_whitespace();

        mock_storage.write().unwrap().expect_get_sec_key().times(1)
            .return_const(sec_key);
        let res = confirm_login(mock_storage, &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(ConfirmLoginError::InvalidConfirmationString)));
    }
}
