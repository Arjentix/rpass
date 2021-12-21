use super::{Connector, DeleteMeError, Error, Result, Unauthorized};

/// Authorized session
///
/// Represents state when session is associated with user
#[derive(Debug)]
pub struct Authorized {
    connector: Connector,
}

impl Authorized {
    /// Creates new Authorized with `connector`
    pub(super) fn new(connector: Connector) -> Self {
        Authorized { connector }
    }

    /// Deletes all information about user the session is associated with
    ///
    /// Consumes `self` and returns `Authorized` object on success or `DeleteMeError` with `self`
    /// on failure
    ///
    /// # Errors
    ///
    /// `DeleteMeError::source` field can have the next values:
    ///
    /// * `Io` - if can't write or read bytes to/from server
    /// * `InvalidResponse` - if response isn't UTF-8 encoded
    /// * `Server` - if server response contains error message
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// use rpass::{session, key::Key};
    ///
    /// # fn main() -> std::result::Result<(), Box<dyn Error>> {
    /// let sec_key = Key::from_file("~/key.sec")?;
    /// let mut session = session::Unauthorized::new("127.0.0.1:3747")?;
    /// session = session.login("user", &sec_key)?.delete_me()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn delete_me(mut self) -> std::result::Result<Unauthorized, DeleteMeError> {
        match self.try_delete_me() {
            Ok(()) => Ok(Unauthorized::with_connector(self.connector)),
            Err(err) => Err(DeleteMeError {
                source: err,
                authorized: self,
            }),
        }
    }

    /// Tries to delete information about user the session is associated with
    ///
    /// See [`Authorized::login()`] for details
    fn try_delete_me(&mut self) -> Result<()> {
        self.connector.send_request(String::from("delete_me"))?;

        match self.connector.recv_response()? {
            ok if ok == "Ok" => Ok(()),
            mes => Err(Error::Server { mes }),
        }
    }
}
