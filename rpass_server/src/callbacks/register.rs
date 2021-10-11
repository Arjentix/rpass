use super::{Result, Error, AsyncStorage, ArgIter, utils};
use crate::storage::Key;
use std::str::FromStr;

/// Registers new user in `storage` with username and key taken from `arg_iter`
/// 
/// Performs username validity check
/// 
/// Returns *Ok("Ok")* in success
/// 
/// # Errors
/// 
/// * `EmptyUsername` - if no username was provided
/// * `InvalidUsername` - if username is invalid
/// * `EmptyKey` - if no key was provided
/// * `InvalidKey` - if key is invalid
/// * `StorageError` - if can't create record cause of some error in
/// `storage`
pub fn register(storage: AsyncStorage, arg_iter: ArgIter)
        -> Result<String> {
    let username = arg_iter.next().ok_or(Error::EmptyUsername)?;
    if !utils::is_safe_for_filename(&username) {
        return Err(Error::InvalidUsername(username));
    }

    let key_string = arg_iter.next().ok_or(Error::EmptyKey)?;
    let key = Key::from_str(&key_string)?;

    let mut storage_write = storage.write().unwrap();
    storage_write.add_new_user(&username, &key)?;

    Ok("Ok".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{super::storage, *};
    use mockall::predicate;

    #[test]
    fn test_ok() {
        let mock_storage = AsyncStorage::default();

        const TEST_USER: &'static str = "test_user";
        const KEY_STR: &'static str = "11:11";

        mock_storage.write().unwrap().expect_add_new_user().times(1)
        .with(predicate::eq(TEST_USER),
            predicate::eq(Key::from_str(KEY_STR).unwrap()))
        .returning(|_, _| Ok(()));

        let args = TEST_USER.to_owned() + " " + KEY_STR;
        let mut arg_iter = args.split_whitespace().map(str::to_owned);
        let res = register(mock_storage, &mut arg_iter);
        assert_eq!(res.unwrap(), "Ok");
    }

    #[test]
    fn test_empty_username() {
        let mock_storage = AsyncStorage::default();

        let mut arg_iter = "".split_whitespace().map(str::to_owned);
        let res = register(mock_storage, &mut arg_iter);
        assert!(matches!(res, Err(Error::EmptyUsername)));
    }

    #[test]
    fn test_invalid_username() {
        let mock_storage = AsyncStorage::default();

        const INVALID_USERNAME: &'static str = "_invalid_username_";
        let mut arg_iter = INVALID_USERNAME.split_whitespace()
            .map(str::to_owned);

        let res = register(mock_storage, &mut arg_iter);
        assert!(matches!(res,
            Err(Error::InvalidUsername(username))
            if username == INVALID_USERNAME));
    }

    #[test]
    fn test_empty_key() {
        let mock_storage = AsyncStorage::default();

        let mut arg_iter = "test_user".split_whitespace().map(str::to_owned);
        let res = register(mock_storage, &mut arg_iter);
        assert!(matches!(res, Err(Error::EmptyKey)));
    }

    #[test]
    fn test_invalid_key() {
        let mock_storage = AsyncStorage::default();

        let mut arg_iter = "test_user key".split_whitespace()
            .map(str::to_owned);
        let res = register(mock_storage, &mut arg_iter);
        assert!(matches!(res, Err(Error::InvalidKey(_))));
    }

    #[test]
    fn test_user_already_exists() {
        let mock_storage = AsyncStorage::default();

        mock_storage.write().unwrap().expect_add_new_user().times(1)
            .returning(|_, _|
                Err(storage::Error::UserAlreadyExists("test_user".to_owned())));
        let mut arg_iter = "test_user 11:11".split_whitespace()
            .map(str::to_owned);
        let res = register(mock_storage, &mut arg_iter);
        assert!(matches!(res, Err(Error::StorageError(_))));
    }
}
