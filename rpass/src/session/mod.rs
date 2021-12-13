mod connector;

pub use crate::{error::*, Result};

use crate::key::Key;
#[mockall_double::double]
use connector::Connector;
use enum_as_inner::EnumAsInner;
use std::net::{TcpStream, ToSocketAddrs};

/// Enum representing user session
#[derive(EnumAsInner, Debug)]
pub enum Session {
    Unauthorized(Unauthorized),
    Authorized(Authorized),
}

#[derive(Debug)]
pub struct Unauthorized {
    connector: Connector,
}

#[derive(Debug)]
pub struct Authorized {
    connector: Connector,
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
    pub fn new<A: ToSocketAddrs>(addr: A, pub_key: Key, sec_key: Key) -> Result<Self> {
        let stream = TcpStream::connect(addr).map_err(|_| Error::CantConnectToTheServer())?;
        let connector = Connector::new(stream, pub_key, sec_key)?;
        Ok(Session::Unauthorized(Unauthorized { connector }))
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// use rpass::{session::Session, key::Key};
    ///
    /// # fn main() -> std::result::Result<(), Box<dyn Error>> {
    /// let pub_key = Key::from_file("~/key.pub")?;
    /// let sec_key = Key::from_file("~/key.sec")?;
    /// let mut session = Session::new("127.0.0.1:3747", pub_key, sec_key)?;
    /// session = match session.into_unauthorized().unwrap().login("user") {
    ///     Ok(authorized) => Session::Authorized(authorized),
    ///     Err(login_err) => {
    ///         println!("Login error: {}", login_err);
    ///         Session::Unauthorized(login_err.unauthorized)
    ///     }
    /// };
    /// # Ok(())
    /// # }
    /// ```
    pub fn login(mut self, username: &str) -> std::result::Result<Authorized, LoginError> {
        match self.try_login(username) {
            Ok(()) => Ok(Authorized {
                connector: self.connector,
            }),
            Err(err) => Err(LoginError {
                source: err,
                unauthorized: self,
            }),
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
            return Err(Error::InvalidUsernameOrKey);
        }

        let decrypted_confirmation = self.connector.sec_key().decrypt(&confirmation);
        let encrypted_confirmation = self
            .connector
            .server_pub_key()
            .encrypt(&decrypted_confirmation);

        let confirm_login_request = format!("confirm_login {}", encrypted_confirmation);
        self.connector.send_request(confirm_login_request)?;

        match self.connector.recv_response()?.as_ref() {
            "Ok" => Ok(()),
            _ => Err(Error::InvalidUsernameOrKey),
        }
    }
}

impl Authorized {
    // TODO impl authorized functions
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests for Unauthorized::login()
    mod login {
        use super::*;
        use std::cell::Cell;
        use std::io;
        use std::rc::Rc;

        use mockall::{predicate::*, Predicate};
        use num_bigint::ToBigUint;

        const TEST_USER: &str = "test_user";
        const CONFIRMATION: &str = "confirmation";

        #[test]
        fn test_ok() {
            let (server_pub_key, pub_key, sec_key) = generate_keys();
            let send_request_arg_validator = {
                let expected_confirmation =
                    build_expected_logging_confirmation(&server_pub_key, &sec_key);
                build_send_request_arg_validator_for(expected_confirmation)
            };

            let mut connector = Connector::default();
            expect_keys_for(&mut connector, sec_key, server_pub_key);
            expect_send_request(&mut connector, send_request_arg_validator);
            expect_recv_response(&mut connector, pub_key);

            let unauthorized = Unauthorized { connector };
            unauthorized.login(TEST_USER).unwrap();
        }

        #[test]
        fn test_cant_send_login_request() {
            let mut connector = Connector::default();
            connector
                .expect_send_request()
                .with(eq(format!("login {}", TEST_USER)))
                .times(1)
                .returning(|_| Err(Error::Io(io::Error::new(io::ErrorKind::Other, ""))));

            let unauthorized = Unauthorized { connector };
            assert!(matches!(
                unauthorized.login(TEST_USER),
                Err(LoginError {
                    source: Error::Io(_),
                    ..
                })
            ));
        }

        #[test]
        fn test_cant_recv_login_response() {
            let mut connector = Connector::default();
            connector
                .expect_send_request()
                .with(eq(format!("login {}", TEST_USER)))
                .times(1)
                .returning(|_| Ok(()));
            connector.expect_recv_response().times(1).returning(|| {
                Err(Error::InvalidResponse(
                    String::from_utf8(vec![0, 159]).unwrap_err(),
                ))
            });

            let unauthorized = Unauthorized { connector };
            assert!(matches!(
                unauthorized.login(TEST_USER),
                Err(LoginError {
                    source: Error::InvalidResponse(_),
                    ..
                })
            ));
        }

        #[test]
        fn test_error_in_login_response() {
            let mut connector = Connector::default();
            connector
                .expect_send_request()
                .with(eq(format!("login {}", TEST_USER)))
                .times(1)
                .returning(|_| Ok(()));
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("Error: invalid username")));

            let unauthorized = Unauthorized { connector };
            assert!(matches!(
                unauthorized.login(TEST_USER),
                Err(LoginError {
                    source: Error::InvalidUsernameOrKey,
                    ..
                })
            ));
        }

        #[test]
        fn test_cant_send_confirm_login_request() {
            let (server_pub_key, pub_key, sec_key) = generate_keys();
            let send_response_call_counter = Rc::new(Cell::new(0u8));
            let send_request_arg_validator = {
                let expected_confirmation =
                    build_expected_logging_confirmation(&server_pub_key, &sec_key);
                let validator_counter = send_response_call_counter.clone();

                move |val: &String| {
                    let counter = validator_counter.get();
                    validator_counter.set(counter + 1);

                    if validator_counter.get() == 1u8 {
                        return val == &format!("login {}", TEST_USER);
                    }
                    val == &format!("confirm_login {}", expected_confirmation)
                }
            };

            let mut connector = Connector::default();
            expect_keys_for(&mut connector, sec_key, server_pub_key);
            connector
                .expect_send_request()
                .withf_st(send_request_arg_validator)
                .times(2)
                .returning_st(move |_| match send_response_call_counter.get() {
                    1 => Ok(()),
                    _ => Err(Error::Io(io::Error::new(io::ErrorKind::Other, ""))),
                });
            connector
                .expect_recv_response()
                .times(1)
                .returning(move || Ok(pub_key.encrypt(CONFIRMATION)));

            let unauthorized = Unauthorized { connector };
            assert!(matches!(
                unauthorized.login(TEST_USER),
                Err(LoginError {
                    source: Error::Io(_),
                    ..
                })
            ));
        }

        #[test]
        fn test_cant_recv_confirm_login_response() {
            let (server_pub_key, pub_key, sec_key) = generate_keys();
            let send_request_arg_validator = {
                let expected_confirmation =
                    build_expected_logging_confirmation(&server_pub_key, &sec_key);
                build_send_request_arg_validator_for(expected_confirmation)
            };
            let mut recv_response_call_counter = 0u8;

            let mut connector = Connector::default();
            expect_keys_for(&mut connector, sec_key, server_pub_key);
            expect_send_request(&mut connector, send_request_arg_validator);
            connector
                .expect_recv_response()
                .times(2)
                .returning(move || {
                    if recv_response_call_counter == 0 {
                        recv_response_call_counter += 1;
                        return Ok(pub_key.encrypt(CONFIRMATION));
                    }

                    Err(Error::Io(io::Error::new(io::ErrorKind::Other, "")))
                });

            let unauthorized = Unauthorized { connector };
            assert!(matches!(
                unauthorized.login(TEST_USER),
                Err(LoginError {
                    source: Error::Io(_),
                    ..
                })
            ));
        }

        #[test]
        fn test_error_in_confirm_login_response() {
            let (server_pub_key, pub_key, sec_key) = generate_keys();
            let send_request_arg_validator = {
                let expected_confirmation =
                    build_expected_logging_confirmation(&server_pub_key, &sec_key);
                build_send_request_arg_validator_for(expected_confirmation)
            };
            let mut recv_response_call_counter = 0u8;

            let mut connector = Connector::default();
            expect_send_request(&mut connector, send_request_arg_validator);
            expect_keys_for(&mut connector, sec_key, server_pub_key);
            connector
                .expect_recv_response()
                .times(2)
                .returning(move || {
                    if recv_response_call_counter == 0 {
                        recv_response_call_counter += 1;
                        return Ok(pub_key.encrypt(CONFIRMATION));
                    }

                    Ok(String::from("Error: invalid confirmation string"))
                });

            let unauthorized = Unauthorized { connector };
            assert!(matches!(
                unauthorized.login(TEST_USER),
                Err(LoginError {
                    source: Error::InvalidUsernameOrKey,
                    ..
                })
            ));
        }

        /// Generates server public key and user's public and secret keys
        fn generate_keys() -> (Key, Key, Key) {
            let server_pub_key = Key(11.to_biguint().unwrap(), 22.to_biguint().unwrap());

            // TODO Change next keys initialization to Key::generate_pair() when it
            // will be possible to pass generator
            let pub_key = Key(269.to_biguint().unwrap(), 221.to_biguint().unwrap());
            let sec_key = Key(5.to_biguint().unwrap(), 221.to_biguint().unwrap());

            (server_pub_key, pub_key, sec_key)
        }

        /// Builds confirmation string that is expected to arrive as confirm_login
        /// request
        fn build_expected_logging_confirmation(server_pub_key: &Key, sec_key: &Key) -> String {
            let decrypted_confirmation = server_pub_key.decrypt(CONFIRMATION);
            sec_key.encrypt(&decrypted_confirmation)
        }

        /// Builds predicate to validate Connector::send_request() function during
        /// logging
        fn build_send_request_arg_validator_for(
            expected_confirmation: String,
        ) -> impl Predicate<String> {
            let counter = Cell::new(0u8);
            function(move |val: &String| {
                if counter.get() == 0 {
                    counter.set(counter.get() + 1);
                    return val == &format!("login {}", TEST_USER);
                }
                val == &format!("confirm_login {}", expected_confirmation)
            })
        }

        /// Adds expecting for sec_key() and server_pub_key() for `connector`
        fn expect_keys_for(connector: &mut Connector, sec_key: Key, server_pub_key: Key) {
            connector.expect_sec_key().times(1).return_const(sec_key);
            connector
                .expect_server_pub_key()
                .times(1)
                .return_const(server_pub_key);
        }

        fn expect_send_request<P>(connector: &mut Connector, validator: P)
        where
            P: Predicate<String> + Send + 'static,
        {
            connector
                .expect_send_request()
                .with(validator)
                .times(2)
                .returning(|_| Ok(()));
        }

        /// Adds expecting for recv_response() for `connector`
        fn expect_recv_response(connector: &mut Connector, pub_key: Key) {
            let mut recv_response_call_counter = 0u8;
            connector
                .expect_recv_response()
                .times(2)
                .returning(move || {
                    if recv_response_call_counter == 0 {
                        recv_response_call_counter += 1;
                        return Ok(pub_key.encrypt(CONFIRMATION));
                    }
                    Ok(String::from("Ok"))
                });
        }
    }
}
