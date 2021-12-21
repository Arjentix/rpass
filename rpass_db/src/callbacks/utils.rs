/// Checks if `name` is a *safe* string to be a filename.
/// Valid means:
/// * Not empty
/// * All characters are ascii alphanumeric or `.`, or `@`, or `_`
/// * Contains at least one alphabetic character
/// * Doesn't contains `..`
/// * Doesn't start with `.`, `@` or `_`
/// * Doesn't end with `.`, `@` or `_`
/// * No more than 32 characters in length
pub fn is_safe_for_filename(name: &str) -> bool {
    !(name.is_empty()
        || !name
            .chars()
            .all(|c| char::is_ascii_alphanumeric(&c) || c == '.' || c == '@' || c == '_')
        || !name.chars().any(|c| char::is_ascii_alphabetic(&c))
        || is_contains_two_dots(name)
        || name.starts_with('.')
        || name.starts_with('@')
        || name.starts_with('_')
        || name.ends_with('.')
        || name.ends_with('@')
        || name.ends_with('_')
        || name.len() > 32)
}

fn is_contains_two_dots(s: &str) -> bool {
    s.chars()
        .zip(s.chars().skip(1))
        .any(|(c1, c2)| c1 == '.' && c2 == '.')
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        assert!(!is_safe_for_filename(""));
        assert!(!is_safe_for_filename("Борщ"));
        assert!(!is_safe_for_filename("786.@09"));
        assert!(!is_safe_for_filename("not/a/hacker/seriously"));
        assert!(!is_safe_for_filename("user..name"));
        assert!(!is_safe_for_filename(".user"));
        assert!(!is_safe_for_filename("user."));
        assert!(!is_safe_for_filename("@user"));
        assert!(!is_safe_for_filename("user@"));
        assert!(!is_safe_for_filename("_user"));
        assert!(!is_safe_for_filename("user_"));
        assert!(!is_safe_for_filename(
            &String::from_utf8(vec![b'X'; 33]).unwrap()
        ));

        assert!(is_safe_for_filename("user_404@example.com"));
    }
}
