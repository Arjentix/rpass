use super::{utils, Connector, DeleteMeError, Error, Record, Result, Unauthorized};
use std::cell::RefCell;
use std::str::FromStr;

/// Authorized session
///
/// Represents state when session is associated with user
#[derive(Debug)]
pub struct Authorized {
    connector: RefCell<Connector>,
}

impl Authorized {
    /// Creates new Authorized with `connector`
    pub(super) fn new(connector: Connector) -> Self {
        Authorized {
            connector: RefCell::new(connector),
        }
    }

    /// Add `record` to the storage
    ///
    /// # Errors
    ///
    /// * `InvalidResource` - if record's resource is empty
    /// * `Io` - if can't write or read bytes to/from server
    /// * `InvalidResponseEncoding` - if response isn't UTF-8 encoded
    /// * `Server` - if server response contains error message
    /// * `UnexpectedResponse` - if server responses with unexpected message
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// use rpass::{session, key::Key, record::Record};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> std::result::Result<(), Box<dyn Error>> {
    /// let sec_key = Key::from_file("~/key.sec")?;
    /// let record = Record {
    ///     resource: String::from("example.com"),
    ///     password: String::from("secret"),
    ///     notes: String::from("important notes")
    /// };
    ///
    /// let mut session = session::Unauthorized::new("127.0.0.1:3747").await?;
    /// let mut session = session.login("user", &sec_key).await?;
    /// session.add_record(&record).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_record(&mut self, record: &Record) -> Result<()> {
        Self::check_resource(&record.resource)?;

        let request = format!("new_record {} \"{}\"", record.resource, record.to_string());
        self.connector.get_mut().send_request(request).await?;

        self.read_ok_response().await
    }

    /// Deletes record with `resource` name
    ///
    /// # Errors
    ///
    /// * `InvalidResource` - if `resource` is empty
    /// * `Io` - if can't write or read bytes to/from server
    /// * `InvalidResponseEncoding` - if response isn't UTF-8 encoded
    /// * `Server` - if server response contains error message
    /// * `UnexpectedResponse` - if server responses with unexpected message
    ///
    /// # Example
    ///
    /// ```
    /// use rpass::session::Authorized;
    /// use tokio::io::{AsyncBufReadExt, BufReader, stdin};
    /// use std::error::Error;
    ///
    /// async fn read_resource_and_delete(session: &mut Authorized) -> Result<(), Box<dyn Error>> {
    ///     let resource = {
    ///         let mut buffer = String::new();
    ///         let mut stdin = BufReader::new(stdin());
    ///         stdin.read_line(&mut buffer).await?;
    ///         buffer
    ///     };
    ///
    ///     session.delete_record(&resource).await.map_err(|err| err.into())
    /// }
    /// ```
    pub async fn delete_record(&mut self, resource: &str) -> Result<()> {
        Self::check_resource(resource)?;

        let request = format!("delete_record {}", resource);
        self.connector.get_mut().send_request(request).await?;

        self.read_ok_response().await
    }

    /// Get record with `resource` name
    ///
    /// # Errors
    ///
    /// * `InvalidResource` - if `resource` is empty
    /// * `Io` - if can't write or read bytes to/from server
    /// * `InvalidResponseEncoding` - if response isn't UTF-8 encoded
    /// * `Server` - if server response contains error message
    /// * `UnexpectedResponse` - if server responses with unexpected message
    ///
    /// # Example
    ///
    /// ```
    /// use rpass::session::Authorized;
    /// use std::io;
    /// use std::error::Error;
    ///
    /// async fn read_resource_and_show_record(session: &Authorized) -> Result<(), Box<dyn Error>> {
    ///     let resource = {
    ///         let mut buffer = String::new();
    ///         let mut stdin = io::stdin();
    ///         stdin.read_line(&mut buffer)?;
    ///         buffer
    ///     };
    ///
    ///     let record = session.get_record(resource).await?;
    ///     println!("{}", record.to_string());
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_record(&self, resource: String) -> Result<Record> {
        Self::check_resource(&resource)?;

        let response = {
            let request = format!("show_record {}", resource);
            let mut connector = self.connector.borrow_mut();
            connector.send_request(request).await?;
            utils::read_good_response(&mut connector).await?
        };

        Ok(Record {
            resource,
            ..Record::from_str(&response)?
        })
    }

    /// Get list of all records names
    ///
    /// # Errors
    ///
    /// * `Io` - if can't write or read bytes to/from server
    /// * `InvalidResponseEncoding` - if response isn't UTF-8 encoded
    /// * `Server` - if server response contains error message
    /// * `UnexpectedResponse` - if server responses with unexpected message
    ///
    /// # Example
    ///
    /// ```
    /// use rpass::session::Authorized;
    /// use std::io;
    /// use std::error::Error;
    ///
    /// async fn print_all_records(session: &Authorized) -> Result<(), Box<dyn Error>> {
    ///     let records = session.get_records_list().await?;
    ///     for record in records.into_iter().enumerate() {
    ///         println!("{}: {}", record.0 + 1, record.1);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_records_list(&self) -> Result<Vec<String>> {
        let response = {
            let mut connector = self.connector.borrow_mut();
            connector.send_request(String::from("list_records")).await?;
            utils::read_good_response(&mut connector).await?
        };

        if response == "No records yet" {
            return Ok(vec![]);
        }

        let records = response.split('\n').map(|s| s.to_owned()).collect();
        Ok(records)
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
    /// * `InvalidResponseEncoding` - if response isn't UTF-8 encoded
    /// * `Server` - if server response contains error message
    ///
    /// # Example
    ///
    /// ```
    /// use rpass::session::{Authorized, Unauthorized};
    /// use tokio::io::{AsyncBufReadExt, BufReader, stdin};
    ///
    /// async fn delete_user(authorized: Authorized) -> Result<Unauthorized, Authorized> {
    ///     println!("Deleting user... Are you sure?");
    ///     let mut buffer = String::new();
    ///     let mut stdin = BufReader::new(stdin());
    ///
    ///     if let Err(_) = stdin.read_line(&mut buffer).await {
    ///         return Err(authorized);
    ///     }
    ///
    ///     if buffer == "Yes" {
    ///         return authorized.delete_me().await.map_err(|err| err.authorized);
    ///     }
    ///
    ///     Err(authorized)
    /// }
    /// ```
    pub async fn delete_me(mut self) -> std::result::Result<Unauthorized, DeleteMeError> {
        match self.try_delete_me().await {
            Ok(()) => Ok(Unauthorized::with_connector(self.connector.into_inner())),
            Err(err) => Err(DeleteMeError {
                source: err,
                authorized: self,
            }),
        }
    }

    /// Tries to delete information about user the session is associated with
    ///
    /// See [`Authorized::login()`] for details
    async fn try_delete_me(&mut self) -> Result<()> {
        self.connector
            .get_mut()
            .send_request(String::from("delete_me"))
            .await?;
        self.read_ok_response().await
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

    /// See [`utils::read_ok_response()`]
    async fn read_ok_response(&mut self) -> Result<()> {
        utils::read_ok_response(self.connector.get_mut()).await
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

        #[tokio::test]
        async fn test_ok() {
            let record = build_record();

            let mut connector = Connector::default();
            expect_all_ok(&mut connector, build_request(&record));

            let mut authorized = Authorized::new(connector);
            authorized.add_record(&record).await.unwrap();
        }

        #[tokio::test]
        async fn test_invalid_resource() {
            let record = Record {
                resource: String::default(),
                password: String::from("secret"),
                notes: String::from("notes"),
            };

            let connector = Connector::default();

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.add_record(&record).await,
                Err(Error::InvalidResource { .. })
            ));
        }

        #[tokio::test]
        async fn test_cant_send_request() {
            let record = build_record();

            let mut connector = Connector::default();
            expect_failing_send_request(&mut connector, build_request(&record));

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.add_record(&record).await,
                Err(Error::Io(_))
            ));
        }

        #[tokio::test]
        async fn test_cant_recv_response() {
            let record = build_record();

            let mut connector = Connector::default();
            expect_failing_recv_response(&mut connector, build_request(&record));

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.add_record(&record).await,
                Err(Error::InvalidResponseEncoding(_))
            ));
        }

        #[tokio::test]
        async fn test_unexpected_response() {
            let record = build_record();

            let mut connector = Connector::default();
            expect_ok_send_request(&mut connector, build_request(&record));
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("Record successfully added")));

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.add_record(&record).await,
                Err(Error::UnexpectedResponse { response })
                    if response == "Record successfully added"
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

        #[tokio::test]
        async fn test_ok() {
            let resource = "test_resource";

            let mut connector = Connector::default();
            expect_all_ok(&mut connector, String::from("delete_record test_resource"));

            let mut authorized = Authorized::new(connector);
            authorized.delete_record(resource).await.unwrap();
        }

        #[tokio::test]
        async fn test_invalid_resource() {
            let resource = "";

            let connector = Connector::default();

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_record(resource).await,
                Err(Error::InvalidResource { .. })
            ));
        }

        #[tokio::test]
        async fn test_cant_send_request() {
            let resource = "test_resource";

            let mut connector = Connector::default();
            expect_failing_send_request(
                &mut connector,
                String::from("delete_record test_resource"),
            );

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_record(resource).await,
                Err(Error::Io(_))
            ));
        }

        #[tokio::test]
        async fn test_cant_recv_response() {
            let resource = "test_resource";

            let mut connector = Connector::default();
            expect_failing_recv_response(
                &mut connector,
                String::from("delete_record test_resource"),
            );

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_record(resource).await,
                Err(Error::InvalidResponseEncoding(_))
            ));
        }

        #[tokio::test]
        async fn test_unexpected_response() {
            let resource = "test_resource";

            let mut connector = Connector::default();
            expect_ok_send_request(&mut connector, String::from("delete_record test_resource"));
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("Record successfully deleted")));

            let mut authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_record(resource).await,
                Err(Error::UnexpectedResponse { response })
                    if response == "Record successfully deleted"
            ));
        }
    }

    /// Tests for `Authorized::get_record()`
    mod get_record {
        use super::*;

        #[tokio::test]
        async fn test_ok() {
            let resource = "test_resource";

            let record = Record {
                resource: resource.to_string(),
                password: String::from("secret"),
                notes: String::from("notes"),
            };
            let record_str = record.to_string();

            let mut connector = Connector::default();
            expect_ok_send_request(&mut connector, format!("show_record {}", resource));
            connector
                .expect_recv_response()
                .times(1)
                .return_once(move || Ok(record_str));

            let authorized = Authorized::new(connector);
            assert_eq!(
                authorized.get_record(resource.to_string()).await.unwrap(),
                record
            );
        }

        #[tokio::test]
        async fn test_invalid_resource() {
            let resource = String::default();

            let connector = Connector::default();

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.get_record(resource).await,
                Err(Error::InvalidResource { .. })
            ));
        }

        #[tokio::test]
        async fn test_cant_send_request() {
            let resource = String::from("test_resource");

            let mut connector = Connector::default();
            expect_failing_send_request(&mut connector, format!("show_record {}", resource));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.get_record(resource).await,
                Err(Error::Io(_))
            ));
        }

        #[tokio::test]
        async fn test_cant_recv_response() {
            let resource = String::from("test_resource");

            let mut connector = Connector::default();
            expect_failing_recv_response(&mut connector, format!("show_record {}", resource));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.get_record(resource).await,
                Err(Error::InvalidResponseEncoding(_))
            ));
        }

        #[tokio::test]
        async fn test_error_from_server() {
            let resource = String::from("test_resource");

            let mut connector = Connector::default();
            expect_ok_send_request(&mut connector, format!("show_record {}", resource));
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("Error: no such record")));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.get_record(resource).await,
                Err(Error::Server { mes }) if mes == "no such record"
            ));
        }

        #[tokio::test]
        async fn test_wrong_record_format() {
            let resource = String::from("test_resource");

            let mut connector = Connector::default();
            expect_ok_send_request(&mut connector, format!("show_record {}", resource));
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("secret, notes")));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.get_record(resource).await,
                Err(Error::CantParseRecord(_))
            ));
        }
    }

    /// Tests for `Authorized::get_records_list()`
    mod get_records_list {
        use super::*;

        #[tokio::test]
        async fn test_ok() {
            let mut connector = Connector::default();
            expect_ok_send_request(&mut connector, String::from("list_records"));
            connector
                .expect_recv_response()
                .times(1)
                .return_once(move || Ok(String::from("example.com\ntest.ru\nyoutube.com")));

            let expected_list: Vec<String> = ["example.com", "test.ru", "youtube.com"]
                .into_iter()
                .map(ToOwned::to_owned)
                .collect();

            let authorized = Authorized::new(connector);
            assert_eq!(authorized.get_records_list().await.unwrap(), expected_list);
        }

        #[tokio::test]
        async fn test_no_records() {
            let mut connector = Connector::default();
            expect_ok_send_request(&mut connector, String::from("list_records"));
            connector
                .expect_recv_response()
                .times(1)
                .return_once(move || Ok(String::from("No records yet")));

            let authorized = Authorized::new(connector);
            assert!(authorized.get_records_list().await.unwrap().is_empty());
        }

        #[tokio::test]
        async fn test_cant_send_request() {
            let mut connector = Connector::default();
            expect_failing_send_request(&mut connector, String::from("list_records"));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.get_records_list().await,
                Err(Error::Io(_))
            ));
        }

        #[tokio::test]
        async fn test_cant_recv_response() {
            let mut connector = Connector::default();
            expect_failing_recv_response(&mut connector, String::from("list_records"));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.get_records_list().await,
                Err(Error::InvalidResponseEncoding(_))
            ));
        }

        #[tokio::test]
        async fn test_error_from_server() {
            let mut connector = Connector::default();
            expect_ok_send_request(&mut connector, String::from("list_records"));
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("Error: some internal error")));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.get_records_list().await,
                Err(Error::Server { mes }) if mes == "some internal error"
            ));
        }
    }

    /// Tests for `Authorized::delete_me()`
    mod delete_me {
        use super::*;

        #[tokio::test]
        async fn test_ok() {
            let mut connector = Connector::default();
            expect_all_ok(&mut connector, String::from("delete_me"));

            let authorized = Authorized::new(connector);
            authorized.delete_me().await.unwrap();
        }

        #[tokio::test]
        async fn test_cant_send_request() {
            let mut connector = Connector::default();
            expect_failing_send_request(&mut connector, String::from("delete_me"));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_me().await,
                Err(DeleteMeError {
                    source: Error::Io(_),
                    ..
                })
            ));
        }

        #[tokio::test]
        async fn test_cant_recv_response() {
            let mut connector = Connector::default();
            expect_failing_recv_response(&mut connector, String::from("delete_me"));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_me().await,
                Err(DeleteMeError {
                    source: Error::InvalidResponseEncoding(_),
                    ..
                })
            ));
        }

        #[tokio::test]
        async fn test_unexpected_response() {
            let mut connector = Connector::default();
            expect_ok_send_request(&mut connector, String::from("delete_me"));
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("You were successfully deleted")));

            let authorized = Authorized::new(connector);
            assert!(matches!(
                authorized.delete_me().await,
                Err(DeleteMeError {
                    source: Error::UnexpectedResponse { response },
                    ..
                }) if response == "You were successfully deleted"
            ));
        }
    }

    /// Expect `connector` to have successful `send_request()` with `request` as expected request
    /// and successful `recv_response()`
    fn expect_all_ok(connector: &mut Connector, request: String) {
        expect_ok_send_request(connector, request);
        connector
            .expect_recv_response()
            .times(1)
            .returning(|| Ok(String::from("Ok")));
    }

    /// Expect `connector` to have successful `send_request()` with `request` as expected request
    fn expect_ok_send_request(connector: &mut Connector, request: String) {
        connector
            .expect_send_request()
            .with(eq(request))
            .times(1)
            .returning(|_| Ok(()));
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
        expect_ok_send_request(connector, request);
        connector.expect_recv_response().times(1).returning(|| {
            Err(Error::InvalidResponseEncoding(
                String::from_utf8(vec![0, 159]).unwrap_err(),
            ))
        });
    }
}
