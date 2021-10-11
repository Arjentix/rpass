use super::{Result, Error, AsyncStorage, Session};

pub fn list_records(_storage: AsyncStorage, _session: &Session)
        -> Result<String> {
    Err(Error::UnacceptableRequestAtThisState)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate;

    const TEST_USER: &'static str = "test_user";

    #[test]
    fn test_ok() {
        let mock_storage = AsyncStorage::default();
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };

        mock_storage.write().unwrap().expect_list_records().times(1)
            .with(predicate::eq(TEST_USER))
            .returning(|_| Ok(vec!["first".to_owned(), "second".to_owned()]));
        assert_eq!(list_records(mock_storage, &session).unwrap(),
            "first\nsecond");
    }

    #[test]
    fn test_non_authorized() {
        let mock_storage = AsyncStorage::default();
        let session = Session::default();

        assert!(matches!(list_records(mock_storage, &session),
            Err(Error::UnacceptableRequestAtThisState)));
    }
}
