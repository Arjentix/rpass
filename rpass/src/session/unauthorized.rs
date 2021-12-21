use super::{Authorized, Connector, Error, LoginError, Result};
use crate::key::Key;
use std::net::{TcpStream, ToSocketAddrs};

/// Unauthorized session
///
/// Represents state when session isn't associated with any user
#[derive(Debug)]
pub struct Unauthorized {
    connector: Connector,
}

impl Unauthorized {
    /// Creates new Unauthorized
    ///
    /// Connects to rpass server on `addr`
    ///
    /// # Errors
    ///
    /// * `CantConnectToTheServer` - if can't connect to the server
    /// * `Io` - if can't read bytes from server
    /// * `InvalidResponse` - if response isn't UTF-8 encoded
    /// * `InvalidKey` - if can't parse server key
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr).map_err(|_| Error::CantConnectToTheServer)?;
        let connector = Connector::new(stream)?;
        Ok(Unauthorized { connector })
    }

    /// Creates new Unauthorized directly accepting `connector`
    pub(super) fn with_connector(connector: Connector) -> Self {
        Unauthorized { connector }
    }

    /// Registers new user with `username` and `pub_key`
    ///
    /// # Errors
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
    /// let pub_key = Key::from_file("~/pub.sec")?;
    /// let mut session = session::Unauthorized::new("127.0.0.1:3747")?;
    /// session.register("user", &pub_key)?;
    /// println!("Successfully registered new user");
    /// # Ok(())
    /// # }
    /// ```
    pub fn register(&mut self, username: &str, pub_key: &Key) -> Result<()> {
        let register_request = format!("register {} {}", username, pub_key.to_string());
        self.connector.send_request(register_request)?;

        match self.connector.recv_response()? {
            ok if ok == "Ok" => Ok(()),
            mes => Err(Error::Server { mes }),
        }
    }

    /// Attempts to log in to the server with `username` name.
    /// Uses `sec_key` to prove identity.
    ///
    /// Consumes `self` and returns `Authorized` object on success or `LoginError` with `self` on
    /// failure
    ///
    /// # Errors
    ///
    /// `LoginError::source` field can have the next values:
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
    /// let session = session::Unauthorized::new("127.0.0.1:3747")?;
    /// let session = session.login("user", &sec_key)?;
    /// println!("Successfully logged in");
    /// # Ok(())
    /// # }
    /// ```
    pub fn login(
        mut self,
        username: &str,
        sec_key: &Key,
    ) -> std::result::Result<Authorized, LoginError> {
        match self.try_login(username, sec_key) {
            Ok(()) => Ok(Authorized::new(self.connector)),
            Err(err) => Err(LoginError {
                source: err,
                unauthorized: self,
            }),
        }
    }

    /// Tries to log in to the server
    ///
    /// See [`Unauthorized::login()`] for details
    fn try_login(&mut self, username: &str, sec_key: &Key) -> Result<()> {
        let login_request = format!("login {}", username);
        self.connector.send_request(login_request)?;

        let login_response = self.connector.recv_response()?;
        if login_response.starts_with("Error") {
            return Err(Error::Server {
                mes: login_response,
            });
        }

        let confirmation = sec_key.decrypt(&login_response);
        let encrypted_confirmation = self.connector.server_pub_key().encrypt(&confirmation);

        let confirm_login_request = format!("confirm_login {}", encrypted_confirmation);
        self.connector.send_request(confirm_login_request)?;

        match self.connector.recv_response()? {
            ok if ok == "Ok" => Ok(()),
            mes => Err(Error::Server { mes }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::ToBigUint;

    mod register {
        use super::*;

        use mockall::predicate::*;

        use std::io;

        const TEST_USER: &str = "test_user";

        #[test]
        fn test_ok() {
            let (_, pub_key, _) = generate_keys();
            let mut connector = Connector::default();
            connector
                .expect_send_request()
                .with(eq(format!(
                    "register {} {}",
                    TEST_USER,
                    pub_key.to_string()
                )))
                .times(1)
                .returning(|_| Ok(()));
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("Ok")));

            let mut unauthorized = Unauthorized { connector };
            unauthorized.register(TEST_USER, &pub_key).unwrap();
        }

        #[test]
        fn test_cant_send_request() {
            let (_, pub_key, _) = generate_keys();
            let mut connector = Connector::default();
            connector
                .expect_send_request()
                .with(eq(format!(
                    "register {} {}",
                    TEST_USER,
                    pub_key.to_string()
                )))
                .times(1)
                .returning(|_| Err(Error::Io(io::Error::new(io::ErrorKind::Other, ""))));

            let mut unauthorized = Unauthorized { connector };
            assert!(matches!(
                unauthorized.register(TEST_USER, &pub_key),
                Err(Error::Io(_))
            ));
        }

        #[test]
        fn test_cant_recv_response() {
            let (_, pub_key, _) = generate_keys();
            let mut connector = Connector::default();
            connector
                .expect_send_request()
                .with(eq(format!(
                    "register {} {}",
                    TEST_USER,
                    pub_key.to_string()
                )))
                .times(1)
                .returning(|_| Ok(()));
            connector.expect_recv_response().times(1).returning(|| {
                Err(Error::InvalidResponse(
                    String::from_utf8(vec![0, 159]).unwrap_err(),
                ))
            });

            let mut unauthorized = Unauthorized { connector };
            assert!(matches!(
                unauthorized.register(TEST_USER, &pub_key),
                Err(Error::InvalidResponse(_))
            ));
        }
    }

    /// Tests for Unauthorized::login()
    mod login {
        use super::*;
        use std::cell::Cell;
        use std::io;
        use std::rc::Rc;

        use mockall::{predicate::*, Predicate};

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
            expect_server_pub_key(&mut connector, server_pub_key);
            expect_send_request(&mut connector, send_request_arg_validator);
            expect_recv_response(&mut connector, pub_key);

            let unauthorized = Unauthorized { connector };
            unauthorized.login(TEST_USER, &sec_key).unwrap();
        }

        #[test]
        fn test_cant_send_login_request() {
            let (_, _, sec_key) = generate_keys();
            let mut connector = Connector::default();
            connector
                .expect_send_request()
                .with(eq(format!("login {}", TEST_USER)))
                .times(1)
                .returning(|_| Err(Error::Io(io::Error::new(io::ErrorKind::Other, ""))));

            let unauthorized = Unauthorized { connector };
            assert!(matches!(
                unauthorized.login(TEST_USER, &sec_key),
                Err(LoginError {
                    source: Error::Io(_),
                    ..
                })
            ));
        }

        #[test]
        fn test_cant_recv_login_response() {
            let (_, _, sec_key) = generate_keys();
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
                unauthorized.login(TEST_USER, &sec_key),
                Err(LoginError {
                    source: Error::InvalidResponse(_),
                    ..
                })
            ));
        }

        #[test]
        fn test_error_in_login_response() {
            let (_, _, sec_key) = generate_keys();
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
                unauthorized.login(TEST_USER, &sec_key),
                Err(LoginError {
                    source: Error::Server { .. },
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
            expect_server_pub_key(&mut connector, server_pub_key);
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
                unauthorized.login(TEST_USER, &sec_key),
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
            expect_server_pub_key(&mut connector, server_pub_key);
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
                unauthorized.login(TEST_USER, &sec_key),
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
            expect_server_pub_key(&mut connector, server_pub_key);
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
                unauthorized.login(TEST_USER, &sec_key),
                Err(LoginError {
                    source: Error::Server { .. },
                    ..
                })
            ));
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

        fn expect_server_pub_key(connector: &mut Connector, server_pub_key: Key) {
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

    /// Generates server public key and user's public and secret keys
    fn generate_keys() -> (Key, Key, Key) {
        let server_pub_key = Key(11.to_biguint().unwrap(), 22.to_biguint().unwrap());

        // TODO Change next keys initialization to Key::generate_pair() when it
        // will be possible to pass generator
        let pub_key = Key(269.to_biguint().unwrap(), 221.to_biguint().unwrap());
        let sec_key = Key(5.to_biguint().unwrap(), 221.to_biguint().unwrap());

        (server_pub_key, pub_key, sec_key)
    }
}
