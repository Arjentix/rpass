use super::{Result, Error, Session};

/// Ends user's session.
///
/// Sets `session.is_authorized` to *false*, `session.username` to
/// String::default(), `session.is_ended` to *true*
///
/// # Errors
///
/// * `UnacceptableRequestAtThisState` - if session is already ended
pub fn quit(session: &mut Session) -> Result<String> {
    if session.is_ended {
        return Err(Error::UnacceptableRequestAtThisState);
    }

    session.is_authorized = false;
    session.username = String::default();
    session.user_storage = None;
    session.is_ended = true;

    Ok("Ok".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok() {
        let mut session = Session {
            is_authorized: true,
            .. Session::default()
        };

        assert_eq!(quit(&mut session).unwrap(), "Ok".to_owned());
        assert!(!session.is_authorized);
        assert_eq!(session.username, String::default());
        assert!(session.is_ended);
    }

    #[test]
    fn test_already_ended() {
        let mut session = Session {
            is_ended: true,
            .. Session::default()
        };

        assert!(matches!(quit(&mut session),
            Err(Error::UnacceptableRequestAtThisState)));
    }
}
