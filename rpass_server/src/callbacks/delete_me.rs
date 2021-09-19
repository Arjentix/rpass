use super::{AsyncStorage, Session};

#[derive(thiserror::Error, Debug)]
pub enum DeleteMeError {
    #[error("unable to delete")]
    UnableToDelete(#[from] std::io::Error)
}

/// Deletes current user. Takes *username* from `session` and deletes it in
/// `storage`
/// 
/// # Errors
/// 
/// * `UnableToDelete` - if for some reason user's data can't be deleted
pub fn delete_me(storage: AsyncStorage, session: &mut Session)
        -> Result<String, DeleteMeError> {
    let mut storage_write = storage.write().unwrap();
    storage_write.delete_user(&session.username)?;
    session.is_authorized = false;
    Ok("Ok".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use mockall::predicate;

    const TEST_USER: &'static str = "test_user";

    #[test]
    fn test_ok() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::default();
        session.username = TEST_USER.to_owned();

        mock_storage.write().unwrap().expect_delete_user()
            .with(predicate::eq(TEST_USER)).returning(|_|Ok(()));
        let res = delete_me(mock_storage, &mut session);
        assert_eq!(res.unwrap(), "Ok");
        assert!(!session.is_authorized);
    }

    #[test]
    fn test_err() {
        let mock_storage = AsyncStorage::default();
        let mut session = Session::default();
        session.username = TEST_USER.to_owned();

        mock_storage.write().unwrap().expect_delete_user()
            .with(predicate::eq(TEST_USER))
            .returning(|_|Err(io::Error::new(io::ErrorKind::Other, "")));
        assert!(delete_me(mock_storage, &mut session).is_err());
    }
}
