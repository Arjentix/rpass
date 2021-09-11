
pub mod errors;

use std::sync::{Arc, RwLock};
use std::str::FromStr;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use crate::storage::{Storage, Key};
use crate::request_dispatcher::{ArgIter};
use crate::session::Session;

use errors::*;

type AsyncStorage = Arc<RwLock<Storage>>;

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

/// First part of user logging. Reads username from `arg_iter`, gets his key
/// from `storage` and writes random encrypted string into
/// `session.login_confirmation`. Returns *Ok() with login confirmation* in success
/// 
/// The next step user should decrypt that random confirmation string,
/// encrypt if with storage public key and send it back.
/// 
/// See `confirm_login` function for second part
/// 
/// # Errors
/// 
/// * `EmptyUsername` - if no username was provided
/// * `NoSuchUser` - if user with such username doesn't exist
pub fn login(storage: AsyncStorage, session: &mut Session, arg_iter: ArgIter)
        -> Result<String, LoginError> {
    let username = arg_iter.next().ok_or(LoginError::EmptyUsername)?;

    let user_pub_key = {
        let storage_read = storage.read().unwrap();
        storage_read.get_user_pub_key(username)?
    };

    const RAND_STRING_LENGTH: usize = 30;
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(RAND_STRING_LENGTH)
        .map(char::from)
        .collect();

    session.login_confirmation = Some(user_pub_key.encrypt(&rand_string));
    session.is_authorized = false;
    Ok(session.login_confirmation.as_ref().unwrap().clone())
}

/// Second and final part of user logging. Reads encrypted confirmation string
/// from `arg_iter`, decrypts it with `storage.sec_key` and checks if it is equal to the
/// `session.login_confirmation`.
/// 
/// Sets `session.is_authorized` to *true* and returns *Ok("Ok")* if everything
/// is good
/// 
/// See `confirm_login` function for second part
/// 
/// # Errors
/// 
/// * `UnacceptableRequestAtThisState` - if there isn't *login_confirmation* in `session` or
/// user already authorized
/// * `EmptyConfirmationString` - if confirmation string wasn't provided
pub fn confirm_login(storage: AsyncStorage, session: &mut Session, arg_iter: ArgIter)
        -> Result<String, ConfirmLoginError> {
    if session.login_confirmation.is_none() || session.is_authorized {
        return Err(ConfirmLoginError::UnacceptableRequestAtThisState);
    }

    let encrypted_confirmation = arg_iter.next()
        .ok_or(ConfirmLoginError::EmptyConfirmationString)?;

    let sec_key;
    {
        let storage_read = storage.read().unwrap();
        sec_key = storage_read.get_sec_key().clone();
    }

    let confirmation = sec_key.decrypt(encrypted_confirmation);
    if &confirmation != session.login_confirmation.as_ref().unwrap() {
        return Err(ConfirmLoginError::EmptyConfirmationString);
    }
    
    session.login_confirmation = None;
    session.is_authorized = true;
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
}
