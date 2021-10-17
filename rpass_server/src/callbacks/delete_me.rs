use super::{Result, Error, AsyncStorage, Session};

/// Deletes current user. Takes *username* from `session` and deletes it in
/// `storage`
/// 
/// # Errors
/// 
/// * `UnacceptableRequestAtThisState` - if not `session.is_authorized`
/// * `StorageError` - if can't create record cause of some error in
/// `storage`
pub fn delete_me(storage: AsyncStorage, session: &mut Session)
        -> Result<String> {
    if !session.is_authorized {
        return Err(Error::UnacceptableRequestAtThisState);
    }

    let mut storage_write = storage.write().unwrap();
    storage_write.delete_user(&session.username)?;
    session.is_authorized = false;
    Ok("Ok".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{super::storage, *};
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
    }

    #[test]
    fn test_non_authorized() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::default();

        assert!(matches!(delete_me(mock_storage, &mut session),
            Err(Error::UnacceptableRequestAtThisState)));
    }

    #[test]
    fn test_storage_error() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session {
            username : TEST_USER.to_owned(),
            .. Session::default()
        };

        mock_storage.write().unwrap().expect_delete_user()
            .with(predicate::eq(TEST_USER))
            .returning(|_|
                Err(
                    storage::Error::IoError(
                        io::Error::new(io::ErrorKind::Other, "")
                    )
                )
            );
        assert!(delete_me(mock_storage, &mut session).is_err());
    }
}
