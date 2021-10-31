use super::{storage, Result, Error, Session, ArgIter, utils};
use std::str::FromStr;

/// Adds new record for user `session.username`.
/// Reads resource name and record (See [`Record::from_str()`]) from `arg_iter`
///
/// # Errors
///
/// * `UnacceptableRequestAtThisState` - if not `session.is_authorized`
/// * `EmptyResourceName` - if resource name wasn't provided
/// * `InvalidResourceName` - if resource name is invalid
/// * `EmptyRecordContent` - if record wasn't provided
/// * `InvalidRecordFormat` - if can't parse *Record*
/// * `Storage` - if can't create record cause of some error in
/// `session.user_storage`
pub fn new_record(session: &Session, arg_iter: ArgIter)
        -> Result<String> {
    if !session.is_authorized {
        return Err(Error::UnacceptableRequestAtThisState);
    }

    let resource = arg_iter.next().ok_or(Error::EmptyResourceName)?;
    if !utils::is_safe_for_filename(&resource) {
        return Err(Error::InvalidResourceName);
    }

    let record = storage::Record {
        resource, ..
        storage::Record::from_str(
            &arg_iter.next().ok_or(Error::EmptyRecordContent)?)?
    };

    let mut storage_write = session.user_storage.as_ref().unwrap()
        .write().unwrap();
    storage_write.write_record(&record)?;
    Ok("Ok".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{storage, AsyncUserStorage};
    use std::io;
    use mockall::predicate;

    const TEST_USER: &str = "test_user";
    const RESOURCE: &str = "example.com";
    const PASSWORD: &str = "secret";
    const NOTES: &str = "first notes\n\"second notes\"\n\"";

    #[test]
    fn test_ok() {
        let content = String::from(PASSWORD) + "\n" + NOTES;

        let expected_record = storage::Record {
            resource: RESOURCE.to_owned(),
            password: PASSWORD.to_owned(),
            notes: NOTES.to_owned()
        };

        let mock_storage = AsyncUserStorage::default();
        mock_storage.write().unwrap().expect_write_record().times(1)
            .with(predicate::eq(expected_record)).returning(|_| Ok(()));
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            user_storage: Some(mock_storage),
            .. Session::default()
        };
        let args = [RESOURCE.to_owned(), content];
        let mut arg_iter = args.iter().cloned();

        assert_eq!(new_record(&session, &mut arg_iter).unwrap(),
            "Ok".to_owned());
    }

    #[test]
    fn test_non_authorized() {
        let content = String::from(PASSWORD) + "\n" + NOTES;

        let session = Session {
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [RESOURCE.to_owned(), content];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(new_record(&session, &mut arg_iter),
            Err(Error::UnacceptableRequestAtThisState)));
    }

    #[test]
    fn test_empty_resource() {
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(new_record(&session, &mut arg_iter),
            Err(Error::EmptyResourceName)));
    }

    #[test]
    fn test_invalid_resource() {
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = ["../illegal/resource/name".to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(new_record(&session, &mut arg_iter),
            Err(Error::InvalidResourceName)));
    }

    #[test]
    fn test_empty_record_content() {
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [RESOURCE.to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(new_record(&session, &mut arg_iter),
            Err(Error::EmptyRecordContent)));
    }

    #[test]
    fn test_invalid_record_format() {
        let content = String::from(PASSWORD);
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [RESOURCE.to_owned(), content];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(new_record(&session, &mut arg_iter),
            Err(Error::InvalidRecordFormat(_))));
    }

    #[test]
    fn test_storage_error() {
        let content = String::from(PASSWORD) + "\n" + NOTES;

        let expected_record = storage::Record {
            resource: RESOURCE.to_owned(),
            password: PASSWORD.to_owned(),
            notes: NOTES.to_owned()
        };

        let mock_storage = AsyncUserStorage::default();
        mock_storage.write().unwrap().expect_write_record().times(1)
            .with(predicate::eq(expected_record))
            .returning(|_|
                Err(storage::Error::Io(
                    io::Error::new(io::ErrorKind::Other, ""))
                )
            );
        let session = Session {
            is_authorized: true,
            username: TEST_USER.to_owned(),
            user_storage: Some(mock_storage),
            .. Session::default()
        };
        let args = [RESOURCE.to_owned(), content];
        let mut arg_iter = args.iter().cloned();
        assert!(matches!(new_record(&session, &mut arg_iter),
            Err(Error::Storage(_))));
    }
}
