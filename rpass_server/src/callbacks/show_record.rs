use super::{storage, AsyncStorage, Session, ArgIter};

/// Shows record for resource from `arg_iter` for user `session.username`
/// 
/// # Errors
/// 
/// * `UnacceptableRequestAtThisState` - if not `session.is_authorized`
/// * `EmptyResourceName` - if resource name wasn't provided
/// * `StorageError` - if can't retrieve record cause of some error in
/// `storage`
pub fn show_record(storage: AsyncStorage, session: &Session, arg_iter: ArgIter)
        -> Result<String, ShowRecordError> {
    if !session.is_authorized {
        return Err(ShowRecordError::UnacceptableRequestAtThisState);
    }

    let resource = arg_iter.next().ok_or(ShowRecordError::EmptyResourceName)?;
    let storage_read = storage.read().unwrap();
    let record = storage_read.get_record(&session.username, &resource)?;
    Ok(record.to_string())
}

#[derive(thiserror::Error, Debug)]
pub enum ShowRecordError {
    #[error("unacceptable request at this state")]
    UnacceptableRequestAtThisState,

    #[error("empty resource name")]
    EmptyResourceName,

    #[error("storage error: {0}")]
    StorageError(#[from] storage::Error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use mockall::predicate;

    const TEST_USER: &'static str = "test_user";
    const TEST_RESOURCE: &'static str = "example.com";

    #[test]
    fn test_ok() {
        let mock_storage = AsyncStorage::default();
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [TEST_RESOURCE.to_owned()];
        let mut arg_iter = args.iter().cloned();

        mock_storage.write().unwrap().expect_get_record().times(1)
            .with(predicate::eq(TEST_USER), predicate::eq(TEST_RESOURCE))
            .returning(|_, _| Ok(storage::Record::default()));
        assert!(show_record(mock_storage, &session, &mut arg_iter).is_ok());
    }

    #[test]
    fn test_non_authorized() {
        let mock_storage = AsyncStorage::default();
        let session = Session {
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [TEST_RESOURCE.to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(show_record(mock_storage, &session, &mut arg_iter),
            Err(ShowRecordError::UnacceptableRequestAtThisState)));
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

        assert!(matches!(show_record(mock_storage, &session, &mut arg_iter),
            Err(ShowRecordError::EmptyResourceName)));
    }

    #[test]
    fn test_storage_error() {
        let mock_storage = AsyncStorage::default();
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = [TEST_RESOURCE.to_owned()];
        let mut arg_iter = args.iter().cloned();

        mock_storage.write().unwrap().expect_get_record().times(1)
            .with(predicate::eq(TEST_USER), predicate::eq(TEST_RESOURCE))
            .returning(|_, _| 
                Err(
                    storage::Error::RecordParsingError(
                        <storage::Record as FromStr>::Err::EmptyString
                    )
                )
            );
        assert!(matches!(show_record(mock_storage, &session, &mut arg_iter),
            Err(ShowRecordError::StorageError(_))));
    }
}
