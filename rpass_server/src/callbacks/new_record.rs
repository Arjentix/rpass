use super::{storage, AsyncStorage, Session, ArgIter};
use std::str::FromStr;

/// Adds new record for user `session.username`.
/// Reads resource name and record (See [`Record::from_str()`]) from `arg_iter`
/// 
/// # Errors
/// 
/// * `UnacceptableRequestAtThisState` - if not `session.is_authorized`
/// * `EmptyResourceName` - if resource name wasn't provided
/// * `EmptyRecordContent` - if record wasn't provided
/// * `InvalidRecordFormat` - if can't parse *Record*
/// * `CantCreateRecord` - if can't create record cause of some error in
/// `storage`
pub fn new_record(storage: AsyncStorage, session: &Session, arg_iter: ArgIter)
        -> Result<String, NewRecordError> {
    if !session.is_authorized {
        return Err(NewRecordError::UnacceptableRequestAtThisState);
    }

    let resource = arg_iter.next().ok_or(NewRecordError::EmptyResourceName)?
        .to_owned();
    let record = storage::Record {
        resource, ..
        storage::Record::from_str(
            &arg_iter.next().ok_or(NewRecordError::EmptyRecordContent)?)?
    };

    let mut storage_write = storage.write().unwrap();
    storage_write.write_record(&session.username, &record)?;
    Ok("Ok".to_owned())
}

#[derive(thiserror::Error, Debug)]
pub enum NewRecordError {
    #[error("unacceptable request at this state")]
    UnacceptableRequestAtThisState,

    #[error("empty resource name")]
    EmptyResourceName,

    #[error("empty record content")]
    EmptyRecordContent,

    #[error("invalid record format")]
    InvalidRecordFormat(#[from] <storage::Record as FromStr>::Err),

    #[error("storage error: {0}")]
    StorageError(#[from] storage::Error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use mockall::predicate;

    const TEST_USER: &'static str = "test_user";
    const RESOURCE: &'static str = "example.com";
    const PASSWORD: &'static str = "secret";
    const NOTES: &'static str = "first notes\n\"second notes\"\n\"";

    #[test]
    fn test_ok() {
        let content = String::from(PASSWORD) + "\n" + NOTES;

        let expected_record = storage::Record {
            resource: RESOURCE.to_owned(),
            password: PASSWORD.to_owned(),
            notes: NOTES.to_owned()
        };
        let mock_storage = AsyncStorage::default();
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [RESOURCE.to_owned(), content];
        let mut arg_iter = args.iter().cloned();

        mock_storage.write().unwrap().expect_write_record().times(1)
            .with(predicate::eq(TEST_USER), predicate::eq(expected_record))
            .returning(|_, _| Ok(()));
        assert_eq!(new_record(mock_storage, &session, &mut arg_iter).unwrap(),
            "Ok".to_owned());
    }

    #[test]
    fn test_non_authorized() {
        let content = String::from(PASSWORD) + "\n" + NOTES;

        let mock_storage = AsyncStorage::default();
        let session = Session {
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [RESOURCE.to_owned(), content];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(new_record(mock_storage, &session, &mut arg_iter),
            Err(NewRecordError::UnacceptableRequestAtThisState)));
    }

    #[test]
    fn test_empty_resource() {
        let mock_storage = AsyncStorage::default();
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(new_record(mock_storage, &session, &mut arg_iter),
            Err(NewRecordError::EmptyResourceName)));
    }

    #[test]
    fn test_empty_record_content() {
        let mock_storage = AsyncStorage::default();
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [RESOURCE.to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(new_record(mock_storage, &session, &mut arg_iter),
            Err(NewRecordError::EmptyRecordContent)));
    }

    #[test]
    fn test_invalid_record_format() {
        let content = String::from(PASSWORD);
        let mock_storage = AsyncStorage::default();
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [RESOURCE.to_owned(), content];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(new_record(mock_storage, &session, &mut arg_iter),
            Err(NewRecordError::InvalidRecordFormat(_))));
    }

    #[test]
    fn test_storage_error() {
        let content = String::from(PASSWORD) + "\n" + NOTES;

        let expected_record = storage::Record {
            resource: RESOURCE.to_owned(),
            password: PASSWORD.to_owned(),
            notes: NOTES.to_owned()
        };
        let mock_storage = AsyncStorage::default();
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [RESOURCE.to_owned(), content];
        let mut arg_iter = args.iter().cloned();

        mock_storage.write().unwrap().expect_write_record().times(1)
            .with(predicate::eq(TEST_USER), predicate::eq(expected_record))
            .returning(|_, _|
                Err(storage::Error::IoError(
                    io::Error::new(io::ErrorKind::Other, ""))
                )
            );
        assert!(matches!(new_record(mock_storage, &session, &mut arg_iter),
            Err(NewRecordError::StorageError(_))));
    }
}
