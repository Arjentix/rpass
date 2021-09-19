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
    if !is_valid_username(username) {
        return Err(RegistrationError::InvalidUsername(username.to_owned()));
    }

    let key_string = arg_iter.next().ok_or(RegistrationError::EmptyKey)?;
    let key = Key::from_str(key_string)?;

    let mut storage_write = storage.write().unwrap();
    storage_write.add_new_user(&username, &key)?;

    Ok("Ok".to_owned())
}

/// Checks if `username` is a valid string. 
/// Valid means:
/// * Not empty
/// * All characters are ascii alphanumeric or `.` or `@`
/// * Contains at least one alphabetic character
/// * Doesn't contains `..`
/// * Doesn't start with `.` or `@`
/// * Doesn't end with `.` or `@`
/// * No more than 32 characters in length
fn is_valid_username(username: &str) -> bool {
    if username.is_empty() ||
        !is_all_ascii_alphanumeric_or_dot_or_at_sign(username) ||
        !username.chars().any(|c| char::is_ascii_alphabetic(&c)) ||
        is_contains_two_dots(username) ||
        username.starts_with('.') || username.starts_with('@') ||
        username.ends_with('.') || username.ends_with('@') ||
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

fn is_all_ascii_alphanumeric_or_dot_or_at_sign(s: &str) -> bool {
    s.chars()
        .all(|c| char::is_ascii_alphanumeric(&c) || c == '.' || c == '@')
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::io;
    // use mockall::predicate;

    // const TEST_USER: &'static str = "test_user";

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
        assert!(!is_valid_username(
            &String::from_utf8(vec![b'X'; 33]).unwrap()));

        assert!(is_valid_username("user404@example.com"));
    }

    #[test]
    fn test_ok() {
        // let mock_storage = AsyncStorage::default();
        // let mut session = Session::default();
        // session.username = TEST_USER.to_owned();

        // mock_storage.write().unwrap().expect_delete_user()
        //     .with(predicate::eq(TEST_USER)).returning(|_|Ok(()));
        // let res = delete_me(mock_storage, &mut session);
        // assert!(res.is_ok());
        // assert_eq!(res.unwrap(), "Ok".to_owned());
    }
}
