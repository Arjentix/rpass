use super::{AsyncStorage, ArgIter};
use crate::storage::Key;
use std::str::FromStr;

#[derive(thiserror::Error, Debug)]
pub enum RegistrationError {
    #[error("empty username")]
    EmptyUsername,

    #[error("invalid username: {0}")]
    InvalidUsername(String),

    #[error("empty key")]
    EmptyKey,

    #[error("invalid key: `{0}`")]
    InvalidKey(#[from] rpass::key::ParseBigIntError),

    #[error("user already exists")]
    AlreadyExists(#[from] std::io::Error)
}

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
/// * `AlreadyExists` - if user with such username already exists
pub fn register(storage: AsyncStorage, arg_iter: ArgIter)
        -> Result<String, RegistrationError> {
    let username = arg_iter.next().ok_or(RegistrationError::EmptyUsername)?;
    if !is_valid_username(&username) {
        return Err(RegistrationError::InvalidUsername(username));
    }

    let key_string = arg_iter.next().ok_or(RegistrationError::EmptyKey)?;
    let key = Key::from_str(&key_string)?;

    let mut storage_write = storage.write().unwrap();
    storage_write.add_new_user(&username, &key)?;

    Ok("Ok".to_owned())
}

/// Checks if `username` is a valid string. 
/// Valid means:
/// * Not empty
/// * All characters are ascii alphanumeric or `.`, or `@`, or `_`
/// * Contains at least one alphabetic character
/// * Doesn't contains `..`
/// * Doesn't start with `.`, `@` or `_`
/// * Doesn't end with `.`, `@` or `_`
/// * No more than 32 characters in length
fn is_valid_username(username: &str) -> bool {
    if username.is_empty() ||
        !username.chars().all(
            |c| char::is_ascii_alphanumeric(&c) || c == '.' || c == '@' ||
            c == '_') ||
        !username.chars().any(|c| char::is_ascii_alphabetic(&c)) ||
        is_contains_two_dots(username) ||
        username.starts_with('.') || username.starts_with('@') ||
        username.starts_with('_') || username.ends_with('.') ||
        username.ends_with('@') || username.ends_with('_') ||
        username.len() > 32 {
            return false;
    }

    true
}

fn is_contains_two_dots(s: &str) -> bool {
    s.chars()
        .zip(s.chars().skip(1))
        .any(|(c1, c2)| c1 == '.' && c2 == '.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
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
        let mut arg_iter = args.split_whitespace();
        let res = register(mock_storage, &mut arg_iter);
        assert_eq!(res.unwrap(), "Ok");
    }

    #[test]
    fn test_empty_username() {
        let mock_storage = AsyncStorage::default();

        let mut arg_iter = "".split_whitespace();
        let res = register(mock_storage, &mut arg_iter);
        assert!(matches!(res, Err(RegistrationError::EmptyUsername)));
    }

    #[test]
    fn test_invalid_username() {
        let mock_storage = AsyncStorage::default();

        const INVALID_USERNAME: &'static str = "_invalid_username_";
        let mut arg_iter = INVALID_USERNAME.split_whitespace();

        let res = register(mock_storage, &mut arg_iter);
        assert!(matches!(res,
            Err(RegistrationError::InvalidUsername(username))
            if username == INVALID_USERNAME));
    }

    #[test]
    fn test_empty_key() {
        let mock_storage = AsyncStorage::default();

        let mut arg_iter = "test_user".split_whitespace();
        let res = register(mock_storage, &mut arg_iter);
        assert!(matches!(res, Err(RegistrationError::EmptyKey)));
    }

    #[test]
    fn test_invalid_key() {
        let mock_storage = AsyncStorage::default();

        let mut arg_iter = "test_user key".split_whitespace();
        let res = register(mock_storage, &mut arg_iter);
        assert!(matches!(res, Err(RegistrationError::InvalidKey(_))));
    }

    #[test]
    fn test_user_already_exists() {
        let mock_storage = AsyncStorage::default();

        mock_storage.write().unwrap().expect_add_new_user().times(1)
            .returning(|_, _| Err(io::Error::new(io::ErrorKind::AlreadyExists, "")));
        let mut arg_iter = "test_user 11:11".split_whitespace();
        let res = register(mock_storage, &mut arg_iter);
        assert!(matches!(res, Err(RegistrationError::AlreadyExists(_))));
    }

    #[test]
    fn test_is_valid_username() {
        assert!(!is_valid_username(""));
        assert!(!is_valid_username("Борщ"));
        assert!(!is_valid_username("786.@09"));
        assert!(!is_valid_username("not/a/hacker/seriously"));
        assert!(!is_valid_username("user..name"));
        assert!(!is_valid_username(".user"));
        assert!(!is_valid_username("user."));
        assert!(!is_valid_username("@user"));
        assert!(!is_valid_username("user@"));
        assert!(!is_valid_username("_user"));
        assert!(!is_valid_username("user_"));
        assert!(!is_valid_username(
            &String::from_utf8(vec![b'X'; 33]).unwrap()));

        assert!(is_valid_username("user_404@example.com"));
    }
}
