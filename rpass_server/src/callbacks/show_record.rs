use super::{Result, Error, session::*, ArgIter, utils};

/// Shows record for resource from `arg_iter` for user stored in `session`
///
/// # Errors
///
/// * `UnacceptableRequestAtThisState` - if session is not an Authorized
/// variant
/// * `EmptyResourceName` - if resource name wasn't provided
/// * `InvalidResourceName` - if resource name is invalid
/// * `Storage` - if can't retrieve record cause of some error in `user_storage`
/// from `session`
pub fn show_record(session: &Session, arg_iter: ArgIter)
        -> Result<String> {
    let authorized_session = session.as_authorized()
        .ok_or(Error::UnacceptableRequestAtThisState)?;

    let resource = arg_iter.next().ok_or(Error::EmptyResourceName)?;
    if !utils::is_safe_for_filename(&resource) {
        return Err(Error::InvalidResourceName);
    }

    let storage_read = authorized_session.user_storage.read().unwrap();
    let record = storage_read.get_record(&resource)?;
    Ok(record.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{AsyncUserStorage, storage};
    use std::sync::{Arc, RwLock};
    use std::str::FromStr;
    use mockall::predicate;

    const TEST_USER: &str = "test_user";
    const TEST_RESOURCE: &str = "example.com";

    #[test]
    fn test_ok() {
        let mock_user_storage: Arc<RwLock<storage::UserStorage>> = Arc::default();
        mock_user_storage.write().unwrap().expect_get_record().times(1)
            .with(predicate::eq(TEST_RESOURCE))
            .returning(|_| Ok(storage::Record::default()));

        let session = Session::Authorized(Authorized {
            username: TEST_USER.to_owned(),
            user_storage: mock_user_storage
        });
        let args = [TEST_RESOURCE.to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(show_record(&session, &mut arg_iter).is_ok());
    }

    #[test]
    fn test_non_authorized() {
        let session = Session::default();

        let args = [TEST_RESOURCE.to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(show_record(&session, &mut arg_iter),
            Err(Error::UnacceptableRequestAtThisState)));
    }

    #[test]
    fn test_empty_resource() {
        let session = Session::Authorized(Authorized {
            username: TEST_USER.to_owned(),
            user_storage: AsyncUserStorage::default()
        });
        let args = [];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(show_record(&session, &mut arg_iter),
            Err(Error::EmptyResourceName)));
    }

    #[test]
    fn test_invalid_resource() {
        let session = Session::Authorized(Authorized {
            username: TEST_USER.to_owned(),
            user_storage: AsyncUserStorage::default()
        });
        let args = ["./../resource.com".to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(show_record(&session, &mut arg_iter),
            Err(Error::InvalidResourceName)));
    }

    #[test]
    fn test_storage_error() {
        let mock_user_storage: Arc<RwLock<storage::UserStorage>> = Arc::default();
        mock_user_storage.write().unwrap().expect_get_record().times(1)
            .with(predicate::eq(TEST_RESOURCE))
            .returning(|_|
                Err(
                    storage::Error::CantParseRecord(
                        <storage::Record as FromStr>::Err::EmptyString
                    )
                )
            );
        let session = Session::Authorized(Authorized {
            username: TEST_USER.to_owned(),
            user_storage: mock_user_storage
        });
        let args = [TEST_RESOURCE.to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(show_record(&session, &mut arg_iter),
            Err(Error::Storage(_))));
    }
}
