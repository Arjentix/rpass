use super::{Result, Error, AsyncStorage, Session};

/// Deletes current user. Takes *username* from `session` and deletes it in
/// `storage`
///
/// # Errors
///
/// * `UnacceptableRequestAtThisState` - if not `session.is_authorized`
/// * `Storage` - if can't delete user cause of some error in `storage`
pub fn delete_me(storage: AsyncStorage, session: &mut Session)
        -> Result<String> {
    if !session.is_authorized {
        return Err(Error::UnacceptableRequestAtThisState);
    }

    session.user_storage = None;
    let mut storage_write = storage.write().unwrap();
    if let Err(err) = storage_write.delete_user(&session.username) {
        session.user_storage = Some(
            storage_write.get_user_storage(&session.username).unwrap()
        );
        return Err(err.into());
    }

    session.is_authorized = false;
    session.user_storage = None;
    Ok("Ok".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{storage, AsyncUserStorage};
    use std::io;
    use mockall::predicate;

    const TEST_USER: &str = "test_user";

    #[test]
    fn test_ok() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            username: TEST_USER.to_owned(),
            is_authorized: true,
            .. Session::default()
        };

        mock_storage.write().unwrap().expect_delete_user()
            .with(predicate::eq(TEST_USER)).returning(|_|Ok(()));
        let res = delete_me(mock_storage, &mut session);
        assert_eq!(res.unwrap(), "Ok");
        assert!(!session.is_authorized);
        assert!(session.user_storage.is_none());
    }

    #[test]
    fn test_non_authorized() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::default();

        assert!(matches!(delete_me(mock_storage, &mut session),
            Err(Error::UnacceptableRequestAtThisState)));
    }

    #[test]
    fn test_multi_session() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            username: TEST_USER.to_owned(),
            user_storage: Some(AsyncUserStorage::default()),
            .. Session::default()
        };

        {
            let mut mock_storage_write = mock_storage.write().unwrap();
            mock_storage_write.expect_delete_user()
            .with(predicate::eq(TEST_USER))
            .returning(|_|
                Err(storage::Error::UnsupportedActionForMultiSession)
            );
            mock_storage_write.expect_get_user_storage()
            .with(predicate::eq(TEST_USER))
            .returning(|_| Ok(AsyncUserStorage::default()));
        }
        assert!(delete_me(mock_storage, &mut session).is_err());
        assert!(session.user_storage.is_some());
    }

    #[test]
    #[should_panic]
    fn test_double_storage_error() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            username : TEST_USER.to_owned(),
            user_storage: Some(AsyncUserStorage::default()),
            .. Session::default()
        };

        {
            let mut mock_storage_write = mock_storage.write().unwrap();
            mock_storage_write.expect_delete_user()
            .with(predicate::eq(TEST_USER))
            .returning(|_|
                Err(storage::Error::UnsupportedActionForMultiSession)
            );
            mock_storage_write.expect_get_user_storage()
            .with(predicate::eq(TEST_USER))
            .returning(|_| Err(
                storage::Error::Io(io::Error::new(io::ErrorKind::Other, "")))
            );
        }
        delete_me(mock_storage, &mut session).unwrap();
    }
}
