mod connector;

pub use crate::{error::*, Result};

use connector::Connector;
use crate::key::Key;
use std::net::{TcpStream, ToSocketAddrs};
use enum_as_inner::EnumAsInner;

/// Enum representing user session
#[derive(EnumAsInner, Debug)]
pub enum Session {
    Unauthorized(Unauthorized),
    Authorized(Authorized)
}

#[derive(Debug)]
pub struct Unauthorized {
    connector: Connector
}

#[derive(Debug)]
pub struct Authorized {
    connector: Connector
}

impl Session {
    /// Creates new Session initialized with **Unauthorized** variant
    ///
    /// Connects to rpass server on `addr` and stores `pub_key` and `sec_key`
    /// for later use
    ///
    /// # Errors
    ///
    /// * `CantConnectToTheServer` - if can't connect to the server
    /// * `Io` - if can't read bytes from server
    /// * `InvalidResponse` - if response isn't UTF-8 encoded
    /// * `InvalidKey` - if can't parse server key
    pub fn new<A: ToSocketAddrs>(
            addr: A, pub_key: Key, sec_key: Key) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .map_err(|_|Error::CantConnectToTheServer())?;
        let connector = Connector::new(stream, pub_key, sec_key)?;
        Ok(Session::Unauthorized(Unauthorized{connector}))
    }
}

impl Unauthorized {
    /// Attempts to log in to the server with `username` name.
    /// Uses keys provided by [`Session::new()`] to decrypt and encrypt messages
    ///
    /// Consumes `self` and returns `Authorized` object on success or `self` on
    /// failure
    ///
    /// # Errors
    ///
    /// `LoginError::source` field can have the next values:
    ///
    /// * `Io` - if can't write or read bytes to/from server
    /// * `InvalidResponse` - if response isn't UTF-8 encoded
    /// * `InvalidUsernameOrKey` - if user with name `username` does not exists
    /// or pub(sec) key(-s) (see [`Session::new()`]) isn't (aren't) valid
    pub fn login(mut self, username: &str)
            -> std::result::Result<Authorized, LoginError> {
        match self.try_login(username) {
            Ok(()) => Ok(Authorized{connector: self.connector}),
            Err(err) => Err(LoginError {
                source: err,
                unauthorized: self
            })
        }
    }

    /// Tries to log in to the server without consuming `self`
    ///
    /// See [`Unauthorized::login()`] for details
    fn try_login(&mut self, username: &str) -> Result<()> {
        let login_request = format!("login {}", username);
        self.connector.send_request(login_request)?;

        let confirmation = self.connector.recv_response()?;
        if confirmation.starts_with("Error") {
            return Err(Error::InvalidUsernameOrKey)
        }

        let decrypted_confirmation = self.connector.sec_key.decrypt(&confirmation);
        let encrypted_confirmation =
            self.connector.server_pub_key.encrypt(&decrypted_confirmation);
        self.connector.send_request(format!("confirm_login {}", encrypted_confirmation))?;

        match self.connector.recv_response()?.as_ref() {
            "Ok" => Ok(()),
            _ => Err(Error::InvalidUsernameOrKey)
        }
    }
}

impl Authorized {
    // TODO impl authorized functions
}
