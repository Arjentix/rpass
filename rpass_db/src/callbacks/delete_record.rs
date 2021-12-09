use super::{Result, Error, session::*, ArgIter, utils};

/// Deletes record for user stored in `session`.
/// Resource name is read from `arg_iter`
///
/// # Errors
///
/// * `UnacceptableRequestAtThisState` - if session is not an Authorized
/// variant
/// * `EmptyResourceName` - if resource name wasn't provided
/// * `InvalidResourceName` - if resource name is invalid
/// * `Storage` - if can't delete record cause of some error in `user_storage`
/// from session
pub fn delete_record(session: &Session, arg_iter: ArgIter) -> Result<String> {
    let authorized_session = session.as_authorized()
        .ok_or(Error::UnacceptableRequestAtThisState)?;

    let resource_name = arg_iter.next().ok_or(Error::EmptyResourceName)?;
    if !utils::is_safe_for_filename(&resource_name) {
        return Err(Error::InvalidResourceName);
    }

    let mut storage_write = authorized_session.user_storage.write().unwrap();
    storage_write.delete_record(&resource_name)?;

    Ok("Ok".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{storage, AsyncUserStorage};
    use std::io;

    use mockall::predicate;

    const TEST_USER: &str = "test_user";
    const TEST_RESOURCE: &str = "example.com";

    #[test]
    fn test_ok() {
        let mock_user_storage = AsyncUserStorage::default();
        mock_user_storage.write().unwrap().expect_delete_record()
            .with(predicate::eq(TEST_RESOURCE))
            .returning(|_| Ok(()));

        let session = Session::Authorized(Authorized {
            username: TEST_USER.to_owned(),
            user_storage: mock_user_storage
        });


        let args = [TEST_RESOURCE.to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(delete_record(&session, &mut arg_iter).is_ok());
    }

    #[test]
    fn test_non_authorized() {
        let session = Session::default();

        let args = [TEST_RESOURCE.to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(delete_record(&session, &mut arg_iter),
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

        assert!(matches!(delete_record(&session, &mut arg_iter),
            Err(Error::EmptyResourceName)));
    }

    #[test]
    fn test_invalid_resource() {
        let session = Session::Authorized(Authorized {
            username: TEST_USER.to_owned(),
            user_storage: AsyncUserStorage::default()
        });

        let args = ["/etc/passwd".to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(delete_record(&session, &mut arg_iter),
            Err(Error::InvalidResourceName)));
    }

    #[test]
    fn test_storage_error() {
        let mock_user_storage = AsyncUserStorage::default();
        mock_user_storage.write().unwrap().expect_delete_record().times(1)
            .with(predicate::eq(TEST_RESOURCE))
            .returning(|_|
                Err(
                    storage::Error::Io(io::Error::new(io::ErrorKind::Other, ""))
                )
            );

        let session = Session::Authorized(Authorized {
            username: TEST_USER.to_owned(),
            user_storage: mock_user_storage
        });

        let args = [TEST_RESOURCE.to_owned()];
        let mut arg_iter = args.iter().cloned();

        assert!(matches!(delete_record(&session, &mut arg_iter),
            Err(Error::Storage(_))));
    }
}