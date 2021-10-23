use super::{Result, Error, AsyncStorage, Session, ArgIter, utils};

/// Shows record for resource from `arg_iter` for user `session.username`
/// 
/// # Errors
/// 
/// * `UnacceptableRequestAtThisState` - if not `session.is_authorized`
/// * `EmptyResourceName` - if resource name wasn't provided
/// * `InvalidResourceName` - if resource name is invalid
/// * `Storage` - if can't retrieve record cause of some error in
/// `storage`
pub fn show_record(storage: AsyncStorage, session: &Session, arg_iter: ArgIter)
        -> Result<String> {
    if !session.is_authorized {
        return Err(Error::UnacceptableRequestAtThisState);
    }

    let resource = arg_iter.next().ok_or(Error::EmptyResourceName)?;
    if !utils::is_safe_for_filename(&resource) {
        return Err(Error::InvalidResourceName);
    }

    let storage_read = storage.read().unwrap();
    let record = storage_read.get_record(&session.username, &resource)?;
    Ok(record.to_string())
}

#[cfg(test)]
mod tests {
    use super::{super::storage, *};
    use std::str::FromStr;
    use mockall::predicate;

    const TEST_USER: &str = "test_user";
    const TEST_RESOURCE: &str = "example.com";

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
            Err(Error::UnacceptableRequestAtThisState)));
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
            Err(Error::EmptyResourceName)));
    }

    #[test]
    fn test_invalid_resource() {
        let mock_storage = AsyncStorage::default();
        let session = Session {
            is_authorized: true,
            username : TEST_USER.to_owned(),
            .. Session::default()
        };
        let args = ["./../resource.com".to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(show_record(mock_storage, &session, &mut arg_iter),
            Err(Error::InvalidResourceName)));
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
                    storage::Error::CantParseRecord(
                        <storage::Record as FromStr>::Err::EmptyString
                    )
                )
            );
        assert!(matches!(show_record(mock_storage, &session, &mut arg_iter),
            Err(Error::Storage(_))));
    }
}
