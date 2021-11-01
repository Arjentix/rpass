use super::{Result, Error, AsyncStorage, session::*, ArgIter};

/// Second and final part of user logging. Reads encrypted confirmation string
/// from `arg_iter`, decrypts it with `storage.sec_key` and checks if it is
/// equal to the *login_confirmation* in session.
///
/// If everything is good then:
/// 1. Sets `session` to the [`Authorized`] state
/// 3. Return *Ok("Ok")*
///
/// See [`super::login()`] function for first part
///
/// # Errors
///
/// * `UnacceptableRequestAtThisState` - if session is not an Unauthorized
/// variant
/// * `EmptyConfirmationString` - if confirmation string wasn't provided
/// * `InvalidConfirmationString` - if confirmation string isn't equal to the
/// one stored in `session`
pub fn confirm_login(storage: AsyncStorage, session: &mut Session,
        arg_iter: ArgIter) -> Result<String> {
    let unauthorized_session = session.as_unauthorized()
        .ok_or(Error::UnacceptableRequestAtThisState)?;

    let encrypted_confirmation = arg_iter.next()
        .ok_or(Error::EmptyConfirmationString)?;

    let sec_key = {
        let storage_read = storage.read().unwrap();
        storage_read.get_sec_key().clone()
    };

    let confirmation = sec_key.decrypt(&encrypted_confirmation);
    if confirmation != unauthorized_session.login_confirmation {
        return Err(Error::InvalidConfirmationString);
    }

    let mut storage_write = storage.write().unwrap();
    *session = Session::Authorized(Authorized {
        username: unauthorized_session.username,
        user_storage:
            storage_write.get_user_storage(&unauthorized_session.username)?,
    });
    Ok("Ok".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{AsyncUserStorage, storage};
    use std::sync::Arc;
    use crate::storage::Key;
    use mockall::predicate;

    const TEST_USER: &str = "test_user";

    #[test]
    fn test_ok() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::Unauthorized(Unauthorized {
            username: TEST_USER.to_owned(),
            login_confirmation: String::from("confirmation")
        });
        let (pub_key, sec_key) = Key::generate_pair();
        let encrypted_confirmation = pub_key.encrypt(
            &session.as_unauthorized().unwrap().login_confirmation);
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
        assert!(session.is_authorized());
    }

    #[test]
    fn test_session_is_authorized() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::Authorized(Authorized {
            username: TEST_USER.to_owned(),
            user_storage: AsyncUserStorage::default()
        });

        let mut arg_iter = [""].iter().map(|&s| s.to_owned());

        let res = confirm_login(mock_storage.clone(), &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(Error::UnacceptableRequestAtThisState)));
        assert!(session.is_unauthorized());
    }

    #[test]
    fn test_session_is_ended() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::Ended;

        let mut arg_iter = [""].iter().map(|&s| s.to_owned());

        let res = confirm_login(mock_storage.clone(), &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(Error::UnacceptableRequestAtThisState)));
        assert!(session.is_unauthorized());
    }

    #[test]
    fn test_empty_confirmation_string() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::default();
        let mut arg_iter = [].iter().map(|s: &&str| s.to_string());

        let res = confirm_login(mock_storage, &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(Error::EmptyConfirmationString)));
        assert!(session.is_unauthorized());
    }

    #[test]
    fn test_invalid_confirmation_string() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::Unauthorized(Unauthorized {
            login_confirmation: String::from("confirmation"),
            .. Unauthorized::default()
        });
        let (pub_key, sec_key) = Key::generate_pair();
        let encrypted_confirmation = pub_key.encrypt("wrong_confirmation");
        let mut arg_iter = encrypted_confirmation.split_whitespace().map(str::to_owned);

        mock_storage.write().unwrap().expect_get_sec_key().times(1)
            .return_const(sec_key);
        let res = confirm_login(mock_storage, &mut session, &mut arg_iter);
        assert!(matches!(res,
            Err(Error::InvalidConfirmationString)));
        assert!(session.is_unauthorized());
    }

    #[test]
    fn test_storage_error() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::Unauthorized(Unauthorized {
            username: TEST_USER.to_owned(),
            login_confirmation: String::from("confirmation"),
        });
        let (pub_key, sec_key) = Key::generate_pair();
        let encrypted_confirmation = pub_key.encrypt(
            &session.as_unauthorized().unwrap().login_confirmation);
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
        assert!(session.is_unauthorized());
    }

}
