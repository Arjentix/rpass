use super::{Result, Error, AsyncStorage, Session};

/// Lists all records names for user `session.username`.
/// Names will be delimited by a new line character
/// 
/// # Errors
/// 
/// * `UnacceptableRequestAtThisState` - if not `session.is_authorized`
/// * `StorageError` - if can't create record cause of some error in
/// `storage`
pub fn list_records(storage: AsyncStorage, session: &Session)
        -> Result<String> {
    if !session.is_authorized {
        return Err(Error::UnacceptableRequestAtThisState);
    }

    let record_names = {
        let storage_read = storage.read().unwrap();
        storage_read.list_records(&session.username)?
    };

    Ok(to_string_with_delimiter(&record_names, '\n'))
}

/// Catenates strings from `values` delimiting them with `delimiter`
fn to_string_with_delimiter(values: &[String], delimiter: char) -> String {
    let mut s = String::default();
    for value in values {
        if !s.is_empty() {
            s.push(delimiter);
        }

        s.push_str(value);
    }

    s
}

#[cfg(test)]
mod tests {
    use super::{*, super::storage};
    use mockall::predicate;

    const TEST_USER: &str = "test_user";

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

    #[test]
    fn test_storage_error() {
        let mock_storage = AsyncStorage::default();
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };

        mock_storage.write().unwrap().expect_list_records().times(1)
            .with(predicate::eq(TEST_USER))
            .returning(|_|
                Err(storage::Error::UserDoesNotExist(TEST_USER.to_owned())));
        assert!(matches!(list_records(mock_storage, &session),
            Err(Error::Storage(_))));
    }
}
