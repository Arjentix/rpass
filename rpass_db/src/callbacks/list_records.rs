use super::{session::*, Error, Result};

/// Lists all records names for user stored in `session`.
/// Names will be delimited by a new line character
///
/// # Errors
///
/// * `UnacceptableRequestAtThisState` - if session is not an Authorized
/// variant
/// * `Storage` - if can't list records cause of some error in `user_storage`
/// from session
pub fn list_records(session: &Session) -> Result<String> {
    let authorized_session = session
        .as_authorized()
        .ok_or(Error::UnacceptableRequestAtThisState)?;

    let record_names = {
        let storage_read = authorized_session.user_storage.read().unwrap();
        storage_read.list_records()?
    };

    Ok(to_string_with_delimiter(&record_names, "\n"))
}

/// Catenates strings from `values` delimiting them with `delimiter`
fn to_string_with_delimiter(values: &[String], delimiter: &str) -> String {
    match !values.is_empty() {
        true => values
            .iter()
            .skip(1)
            .fold(values[0].clone(), |acc, s| acc + delimiter + s),
        false => String::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{storage, AsyncUserStorage};
    use super::*;
    use std::io;

    #[test]
    fn test_ok() {
        let mock_user_storage = AsyncUserStorage::default();
        mock_user_storage
            .write()
            .unwrap()
            .expect_list_records()
            .times(1)
            .returning(|| Ok(vec!["first".to_owned(), "second".to_owned()]));
        let session = Session::Authorized(Authorized {
            username: String::default(),
            user_storage: mock_user_storage,
        });

        assert_eq!(list_records(&session).unwrap(), "first\nsecond");
    }

    #[test]
    fn test_empty_list() {
        let mock_user_storage = AsyncUserStorage::default();
        mock_user_storage
            .write()
            .unwrap()
            .expect_list_records()
            .times(1)
            .returning(|| Ok(vec![]));
        let session = Session::Authorized(Authorized {
            username: String::default(),
            user_storage: mock_user_storage,
        });

        assert_eq!(list_records(&session).unwrap(), "");
    }

    #[test]
    fn test_non_authorized() {
        let session = Session::default();

        assert!(matches!(
            list_records(&session),
            Err(Error::UnacceptableRequestAtThisState)
        ));
    }

    #[test]
    fn test_storage_error() {
        let mock_user_storage = AsyncUserStorage::default();
        mock_user_storage
            .write()
            .unwrap()
            .expect_list_records()
            .times(1)
            .returning(|| Err(storage::Error::Io(io::Error::new(io::ErrorKind::Other, ""))));
        let session = Session::Authorized(Authorized {
            username: String::default(),
            user_storage: mock_user_storage,
        });

        assert!(matches!(list_records(&session), Err(Error::Storage(_))));
    }
}
