use super::{Result, Error, Session};

/// Lists all records names for user `session.username`.
/// Names will be delimited by a new line character
///
/// # Errors
///
/// * `UnacceptableRequestAtThisState` - if not `session.is_authorized`
/// * `Storage` - if can't create record cause of some error in
/// `storage`
pub fn list_records(session: &Session)
        -> Result<String> {
    if !session.is_authorized {
        return Err(Error::UnacceptableRequestAtThisState);
    }

    let record_names = {
        let storage_read = session.user_storage.as_ref().unwrap().read().unwrap();
        storage_read.list_records()?
    };

    Ok(to_string_with_delimiter(&record_names, "\n"))
}

/// Catenates strings from `values` delimiting them with `delimiter`
fn to_string_with_delimiter(values: &[String], delimiter: &str) -> String {
    match !values.is_empty() {
        true => values.iter().skip(1)
            .fold(values[0].clone(), |acc, s| acc + delimiter + s),
        false => String::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{storage, AsyncUserStorage};
    use std::io;

    #[test]
    fn test_ok() {
        let mock_user_storage = AsyncUserStorage::default();
        mock_user_storage.write().unwrap().expect_list_records().times(1)
            .returning(|| Ok(vec!["first".to_owned(), "second".to_owned()]));
        let session = Session {
            is_authorized: true,
            user_storage: Some(mock_user_storage),
            .. Session::default()
        };

        assert_eq!(list_records(&session).unwrap(), "first\nsecond");
    }

    #[test]
    fn test_empty_list() {
        let mock_user_storage = AsyncUserStorage::default();
        mock_user_storage.write().unwrap().expect_list_records().times(1)
            .returning(|| Ok(vec![]));
        let session = Session {
            is_authorized: true,
            user_storage: Some(mock_user_storage),
            .. Session::default()
        };

        assert_eq!(list_records(&session).unwrap(), "");
    }

    #[test]
    fn test_non_authorized() {
        let session = Session::default();

        assert!(matches!(list_records(&session),
            Err(Error::UnacceptableRequestAtThisState)));
    }

    #[test]
    fn test_storage_error() {
        let mock_user_storage = AsyncUserStorage::default();
        mock_user_storage.write().unwrap().expect_list_records().times(1)
            .returning(||Err(storage::Error::Io(
                io::Error::new(io::ErrorKind::Other, ""))));
        let session = Session {
            is_authorized: true,
            user_storage: Some(mock_user_storage),
            .. Session::default()
        };

        assert!(matches!(list_records(&session),
            Err(Error::Storage(_))));
    }
}
