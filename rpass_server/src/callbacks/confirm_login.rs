use super::{Result, Error, AsyncStorage, Session, ArgIter};

/// Second and final part of user logging. Reads encrypted confirmation string
/// from `arg_iter`, decrypts it with `storage.sec_key` and checks if it is
/// equal to the `session.login_confirmation`.
///
/// If everything is good then:
/// 1. Sets `session.is_authorized` to *true*
/// 2. Sets `session.user_storage` to *Some(_)*
/// 3. Return *Ok("Ok")*
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
        arg_iter: ArgIter) -> Result<String> {
    if session.login_confirmation.is_none() || session.is_authorized {
        return Err(Error::UnacceptableRequestAtThisState);
    }

    let encrypted_confirmation = arg_iter.next()
        .ok_or(Error::EmptyConfirmationString)?;

    let sec_key;
    {
        let storage_read = storage.read().unwrap();
        sec_key = storage_read.get_sec_key().clone();
    }

    let confirmation = sec_key.decrypt(&encrypted_confirmation);
    if &confirmation != session.login_confirmation.as_ref().unwrap() {
        return Err(Error::InvalidConfirmationString);
    }

    let mut storage_write = storage.write().unwrap();
    session.user_storage = Some(storage_write
        .get_user_storage(&session.username)?);
    session.login_confirmation = None;
    session.is_authorized = true;
    Ok("Ok".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::storage;
    use std::sync::Arc;
    use crate::storage::Key;
    use mockall::predicate;

    const TEST_USER: &str = "test_user";

    #[test]
    fn test_ok() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            login_confirmation: Some(String::from("confirmation")),
            username: TEST_USER.to_owned(),
            .. Session::default()
        };
        let (pub_key, sec_key) = Key::generate_pair();
        let encrypted_confirmation = pub_key.encrypt(
            session.login_confirmation.as_ref().unwrap());
        let mut arg_iter = encrypted_confirmation.split_whitespace().map(str::to_owned);

        {
            let mut mock_storage_write = mock_storage.write().unwrap();
            mock_storage_write.expect_get_sec_key().times(1)
                .return_const(sec_key);
            mock_storage_write.expect_get_user_storage()
                .with(predicate::eq(TEST_USER)).times(1)
                .returning(|_|Ok(Arc::default()));
        }
        let res = confirm_login(mock_storage, &mut session, &mut arg_iter);
        assert_eq!(res.unwrap(), "Ok");
        assert!(session.login_confirmation.is_none());
        assert!(session.is_authorized);
        assert!(session.user_storage.is_some());
    }

    #[test]
    fn test_unacceptable_request_at_this_state() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::default();

        let mut arg_iter = [""].iter().map(|&s| s.to_owned());

        let res = confirm_login(mock_storage.clone(), &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(Error::UnacceptableRequestAtThisState)));

        session.login_confirmation = Some(String::default());
        session.is_authorized = true;
        let res = confirm_login(mock_storage, &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(Error::UnacceptableRequestAtThisState)));
    }

    #[test]
    fn test_empty_confirmation_string() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            login_confirmation: Some(String::default()),
            .. Session::default()
        };
        let mut arg_iter = [].iter().map(|s: &&str| s.to_string());

        let res = confirm_login(mock_storage, &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(Error::EmptyConfirmationString)));
    }

    #[test]
    fn test_invalid_confirmation_string() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            login_confirmation: Some(String::from("confirmation")),
            .. Session::default()
        };
        let (pub_key, sec_key) = Key::generate_pair();
        let encrypted_confirmation = pub_key.encrypt("wrong_confirmation");
        let mut arg_iter = encrypted_confirmation.split_whitespace().map(str::to_owned);

        mock_storage.write().unwrap().expect_get_sec_key().times(1)
            .return_const(sec_key);
        let res = confirm_login(mock_storage, &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(Error::InvalidConfirmationString)));
    }

    #[test]
    fn test_storage_error() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            login_confirmation: Some(String::from("confirmation")),
            username: TEST_USER.to_owned(),
            .. Session::default()
        };
        let (pub_key, sec_key) = Key::generate_pair();
        let encrypted_confirmation = pub_key.encrypt(
            session.login_confirmation.as_ref().unwrap());
        let mut arg_iter = encrypted_confirmation.split_whitespace().map(str::to_owned);

        {
            let mut mock_storage_write = mock_storage.write().unwrap();
            mock_storage_write.expect_get_sec_key().times(1)
                .return_const(sec_key);
            mock_storage_write.expect_get_user_storage()
                .with(predicate::eq(TEST_USER)).times(1)
                .returning(|_|Err(
                    storage::Error::UserAlreadyExists(TEST_USER.to_owned())));
        }
        let res = confirm_login(mock_storage, &mut session, &mut arg_iter);
        assert!(matches!(res, Err(Error::Storage(_))));
        assert!(session.login_confirmation.is_some());
        assert!(!session.is_authorized);
        assert!(session.user_storage.is_none());
    }

}
