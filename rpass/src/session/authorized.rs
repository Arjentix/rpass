use super::{Connector, DeleteMeError, Error, Record, Result, Unauthorized};

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

    /// Add `record` to the storage
    ///
    /// # Errors
    ///
    /// * `InvalidResource` - if record's resource is empty
    /// * `Io` - if can't write or read bytes to/from server
    /// * `InvalidResponse` - if response isn't UTF-8 encoded
    /// * `Server` - if server response contains error message
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// use rpass::{session, key::Key, record::Record};
    ///
    /// # fn main() -> std::result::Result<(), Box<dyn Error>> {
    /// let sec_key = Key::from_file("~/key.sec")?;
    /// let record = Record {
    ///     resource: String::from("example.com"),
    ///     password: String::from("secret"),
    ///     notes: String::from("important notes")
    /// };
    ///
    /// let mut session = session::Unauthorized::new("127.0.0.1:3747")?;
    /// let mut session = session.login("user", &sec_key)?;
    /// session.add_record(&record)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_record(&mut self, record: &Record) -> Result<()> {
        Self::check_resource(&record.resource)?;

        let request = format!("new_record {} \"{}\"", record.resource, record.to_string());
        self.connector.send_request(request)?;

        self.check_response()
    }

    /// Deletes record with `resource` name
    ///
    /// # Errors
    ///
    /// * `InvalidResource` - if `resource` is empty
    /// * `Io` - if can't write or read bytes to/from server
    /// * `InvalidResponse` - if response isn't UTF-8 encoded
    /// * `Server` - if server response contains error message
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rpass::session::Authorized;
    /// use std::io;
    /// use std::error::Error;
    ///
    /// fn read_resource_and_delete(session: &mut Authorized) -> Result<(), Box<dyn Error>> {
    /// let resource = {
    ///     let mut buffer = String::new();
    ///     let mut stdin = io::stdin();
    ///     stdin.read_line(&mut buffer)?;
    ///     buffer
    /// };
    ///
    /// session.delete_record(&resource).map_err(|err| err.into())
    /// }
    /// ```
    pub fn delete_record(&mut self, resource: &str) -> Result<()> {
        Self::check_resource(resource)?;

        let request = format!("delete_record {}", resource);
        self.connector.send_request(request)?;

        self.check_response()
    }

    /// Deletes all information about user the session is associated with
    ///
    /// Consumes `self` and returns `Unauthorized` object on success or `DeleteMeError` with `self`
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
        self.check_response()
    }

    /// Checks if `resource` is empty
    ///
    /// # Errors
    ///
    /// Returns `InvalidResource` if `resource` is empty
    fn check_resource(resource: &str) -> Result<()> {
        if let true = resource.is_empty() {
            return Err(Error::InvalidResource {
                mes: String::from("record's resource can't be empty"),
            });
        }

        Ok(())
    }

    /// Checks if server response contains *"Ok"* value.
    /// If not then returns `Error::Server`
    fn check_response(&mut self) -> Result<()> {
        match self.connector.recv_response()? {
            ok if ok == "Ok" => Ok(()),
            mes => Err(Error::Server { mes }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use std::io;

    /// Tests for `Authorized::add_record()`
    mod add_record {
        use super::*;

        #[test]
        fn test_ok() {
            let record = build_record();

            let mut connector = Connector::default();
            expect_all_ok(&mut connector, build_request(&record));

            let mut authorized = Authorized::new(connector);
            authorized.add_record(&record).unwrap();
        }

        #[test]
        fn test_invalid_resource() {
            let record = Record {
                resource: String::default(),
                password: String::from("secret"),
                notes: String::from("notes"),
            };

            let connector = Connector::default();

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.add_record(&record),
                Err(Error::InvalidResource { .. })
            ));
        }

        #[test]
        fn test_cant_send_request() {
            let record = build_record();

            let mut connector = Connector::default();
            expect_failing_send_request(&mut connector, build_request(&record));

            let mut authorized = Authorized::new(connector);
            assert!(matches!(authorized.add_record(&record), Err(Error::Io(_))));
        }

        #[test]
        fn test_cant_recv_response() {
            let record = build_record();

            let mut connector = Connector::default();
            expect_failing_recv_response(&mut connector, build_request(&record));

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.add_record(&record),
                Err(Error::InvalidResponse(_))
            ));
        }

        /// Builds test record
        fn build_record() -> Record {
            Record {
                resource: String::from("test.com"),
                password: String::from("secret"),
                notes: String::from("important notes"),
            }
        }

        /// Build expected request for `record`
        fn build_request(record: &Record) -> String {
            format!("new_record {} \"{}\"", record.resource, record.to_string())
        }
    }

    /// Tests for `Authorized::delete_record()`
    mod delete_record {
        use super::*;

        #[test]
        fn test_ok() {
            let resource = "test_resource";

            let mut connector = Connector::default();
            expect_all_ok(&mut connector, String::from("delete_record test_resource"));

            let mut authorized = Authorized::new(connector);
            authorized.delete_record(resource).unwrap();
        }

        #[test]
        fn test_invalid_resource() {
            let resource = "";

            let connector = Connector::default();

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_record(resource),
                Err(Error::InvalidResource { .. })
            ));
        }

        #[test]
        fn test_cant_send_request() {
            let resource = "test_resource";

            let mut connector = Connector::default();
            expect_failing_send_request(
                &mut connector,
                String::from("delete_record test_resource"),
            );

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_record(resource),
                Err(Error::Io(_))
            ));
        }

        #[test]
        fn test_cant_recv_response() {
            let resource = "test_resource";

            let mut connector = Connector::default();
            expect_failing_recv_response(
                &mut connector,
                String::from("delete_record test_resource"),
            );

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_record(resource),
                Err(Error::InvalidResponse(_))
            ));
        }
    }
    /// Tests for `Authorized::delete_me()`
    mod delete_me {
        use super::*;

        #[test]
        fn test_ok() {
            let mut connector = Connector::default();
            expect_all_ok(&mut connector, String::from("delete_me"));

            let authorized = Authorized::new(connector);
            authorized.delete_me().unwrap();
        }

        #[test]
        fn test_cant_send_request() {
            let mut connector = Connector::default();
            expect_failing_send_request(&mut connector, String::from("delete_me"));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_me(),
                Err(DeleteMeError {
                    source: Error::Io(_),
                    ..
                })
            ));
        }

        #[test]
        fn test_cant_recv_response() {
            let mut connector = Connector::default();
            expect_failing_recv_response(&mut connector, String::from("delete_me"));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_me(),
                Err(DeleteMeError {
                    source: Error::InvalidResponse(_),
                    ..
                })
            ));
        }
    }

    /// Expect `connector` to have successful `send_request()` with `request` as expected request
    /// and successful `recv_response()`
    fn expect_all_ok(connector: &mut Connector, request: String) {
        connector
            .expect_send_request()
            .with(eq(request))
            .times(1)
            .returning(|_| Ok(()));
        connector
            .expect_recv_response()
            .times(1)
            .returning(|| Ok(String::from("Ok")));
    }

    /// Expect `connector` to accept `request` into `send_request` and then return error
    fn expect_failing_send_request(connector: &mut Connector, request: String) {
        connector
            .expect_send_request()
            .with(eq(request))
            .times(1)
            .returning(|_| Err(Error::Io(io::Error::new(io::ErrorKind::Other, ""))));
    }

    /// Expect `connector` to have successful `send_request()` with `request` as expected request
    /// and `recv_response()` returning error
    fn expect_failing_recv_response(connector: &mut Connector, request: String) {
        connector
            .expect_send_request()
            .with(eq(request))
            .times(1)
            .returning(|_| Ok(()));
        connector.expect_recv_response().times(1).returning(|| {
            Err(Error::InvalidResponse(
                String::from_utf8(vec![0, 159]).unwrap_err(),
            ))
        });
    }
}
