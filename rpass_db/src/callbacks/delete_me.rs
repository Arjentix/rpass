use super::{Result, Error, AsyncStorage, session::*};

/// Deletes current user. Takes *username* from `session` and deletes it in
/// `storage`
///
/// # Errors
///
/// * `UnacceptableRequestAtThisState` - if session is not an Authorized
/// variant
/// * `Storage` - if can't delete user cause of some error in `storage`
pub fn delete_me(storage: AsyncStorage, session: &mut Session)
        -> Result<String> {
    let authorized_session = session.as_authorized()
        .ok_or(Error::UnacceptableRequestAtThisState)?;

    let username = authorized_session.username.clone();
    *session = Session::Unauthorized(Unauthorized::default());

    let mut storage_write = storage.write().unwrap();
    if let Err(err) = storage_write.delete_user(&username) {
        *session = Session::Authorized(Authorized {
            user_storage: storage_write.get_user_storage(&username).unwrap(),
            username
        });
        return Err(err.into());
    }

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
        let mut session = Session::Authorized(Authorized {
            username: TEST_USER.to_owned(),
            user_storage: AsyncUserStorage::default()
        });

        mock_storage.write().unwrap().expect_delete_user()
            .with(predicate::eq(TEST_USER)).returning(|_|Ok(()));
        let res = delete_me(mock_storage, &mut session);
        assert_eq!(res.unwrap(), "Ok");
        assert!(matches!(session, Session::Unauthorized(_)));
        assert!(session.as_unauthorized().unwrap().username.is_empty());
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
        let mut session = Session::Authorized(Authorized {
            username: TEST_USER.to_owned(),
            user_storage: AsyncUserStorage::default()
        });

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
        assert!(matches!(session, Session::Authorized(_)));
    }

    #[test]
    #[should_panic]
    fn test_double_storage_error() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::Authorized(Authorized {
            username: TEST_USER.to_owned(),
            user_storage: AsyncUserStorage::default()
        });

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
        let _ = delete_me(mock_storage, &mut session);
    }
}
