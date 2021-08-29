use std::sync::{Arc, RwLock};

use crate::storage::{Storage, Key};
use crate::request_dispatcher::ArgIter;

use std::str::FromStr;

pub fn register(storage: Arc<RwLock<Storage>>, arg_iter: ArgIter)
        -> String {
    let username = match arg_iter.next() {
        Some(username) => username,
        None => return "Error: empty username".to_string()
    };
    if !is_valid_username(username) {
        return "Error: invalid username".to_string();
    }

    let key_string = match arg_iter.next() {
        Some(key_string) => key_string,
        None => return "Error: empty key".to_string()
    };
    let key = match Key::from_str(key_string) {
        Ok(key) => key,
        Err(err) => return err.to_string()
    };

    let mut storage_write = storage.write().unwrap();
    match storage_write.add_new_user(&username, &key) {
        Ok(()) => "Ok".to_string(),
        Err(err) => err.to_string()
    }
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
