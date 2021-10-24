use super::{Result, Error, AsyncStorage, Session, ArgIter};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

/// First part of user logging. Reads username from `arg_iter`, gets his key
/// from `storage` and writes random encrypted string into
/// `session.login_confirmation`.
/// Returns *Ok() with login confirmation* in success
///
/// The next step user should decrypt that random confirmation string,
/// encrypt if with storage public key and send it back.
///
/// See [`super::confirm_login()`] function for second part
///
/// # Errors
///
/// * `EmptyUsername` - if no username was provided
/// * `Storage` - if can't create record cause of some error in
/// `storage`
pub fn login(storage: AsyncStorage, session: &mut Session, arg_iter: ArgIter)
        -> Result<String> {
    let username = arg_iter.next().ok_or(Error::EmptyUsername)?;

    let user_pub_key = {
        let storage_read = storage.read().unwrap();
        storage_read.get_user_pub_key(&username)?
    };

    const RAND_STRING_LENGTH: usize = 30;
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(RAND_STRING_LENGTH)
        .map(char::from)
        .collect();

    session.login_confirmation = Some(user_pub_key.encrypt(&rand_string));
    session.is_authorized = false;
    session.username = username;
    Ok(session.login_confirmation.as_ref().unwrap().clone())
}

#[cfg(test)]
mod tests {
    use super::{super::storage, *};
    use crate::storage::Key;
    use std::str::FromStr;
    use mockall::predicate;

    const TEST_USER: &str = "test_user";

    #[test]
    fn test_ok() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::default();
        let mut arg_iter = [TEST_USER].iter().map(|&s| s.to_owned());

        mock_storage.write().unwrap().expect_get_user_pub_key().times(1)
            .with(predicate::eq(TEST_USER))
            .returning(|_| Ok(Key::from_str("11:11").unwrap()));

        let res = login(mock_storage, &mut session, &mut arg_iter);
        assert!(res.is_ok());
        assert!(matches!(session.login_confirmation, Some(_)));
        assert!(!session.is_authorized);
        assert_eq!(session.username, TEST_USER);
    }

    #[test]
    fn test_empty_username() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::default();
        let mut arg_iter = [].iter().map(|s: &&str| s.to_string());

        let res = login(mock_storage, &mut session, &mut arg_iter);
        assert!(matches!(res, Err(Error::EmptyUsername)));
    }

    #[test]
    fn test_no_such_user() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::default();
        let mut arg_iter = [TEST_USER].iter().map(|&s| s.to_owned());

        mock_storage.write().unwrap().expect_get_user_pub_key().times(1)
            .with(predicate::eq(TEST_USER))
            .returning(|_| Err(
                storage::Error::UserDoesNotExist(TEST_USER.to_owned())
            ));
        let res = login(mock_storage, &mut session, &mut arg_iter);
        assert!(matches!(res, Err(Error::Storage(_))));
    }
}
