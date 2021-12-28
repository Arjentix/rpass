use super::{Connector, Error, Result};

/// Reads response from with `connector` and returns it if it doesn't contain error message
///
/// # Errors
///
/// * `Io` - if can't write or read bytes to/from server
/// * `InvalidResponseEncoding` - if response isn't UTF-8 encoded
/// * `Server` - if server response contains error message
pub fn read_good_response(connector: &mut Connector) -> Result<String> {
    let response = connector.recv_response()?;

    if let Some(stripped) = response.strip_prefix("Error: ") {
        return Err(Error::Server {
            mes: stripped.to_string(),
        });
    }

    Ok(response)
}

/// Checks if server response contains *"Ok"* value
///
/// # Errors
///
/// * `Io` - if can't write or read bytes to/from server
/// * `InvalidResponseEncoding` - if response isn't UTF-8 encoded
/// * `Server` - if server response contains error message
/// * `UnexpectedResponse` - if response isn't *"Ok"* or error
pub fn read_ok_response(connector: &mut Connector) -> Result<()> {
    let response = read_good_response(connector)?;
    match response {
        ok if ok == "Ok" => Ok(()),
        mes => Err(Error::UnexpectedResponse { response: mes }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod read_good_response {
        use super::*;
        use std::io;

        #[test]
        fn test_ok() {
            let mut connector = Connector::default();
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("Good job")));

            assert_eq!(read_good_response(&mut connector).unwrap(), "Good job")
        }

        #[test]
        fn test_cant_recv_response() {
            let mut connector = Connector::default();
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Err(Error::Io(io::Error::new(io::ErrorKind::Other, ""))));

            assert!(matches!(
                read_good_response(&mut connector),
                Err(Error::Io(_))
            ))
        }

        #[test]
        fn test_response_with_error() {
            let mut connector = Connector::default();
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("Error: test error")));

            assert!(matches!(
                read_good_response(&mut connector),
                Err(Error::Server { mes }) if mes == "test error"
            ))
        }
    }

    mod read_ok_response {
        use super::*;

        #[test]
        fn test_ok() {
            let mut connector = Connector::default();
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("Ok")));

            read_good_response(&mut connector).unwrap();
        }

        #[test]
        fn test_unexpected_response() {
            let mut connector = Connector::default();
            connector
                .expect_recv_response()
                .times(1)
                .returning(|| Ok(String::from("Good")));

            assert!(matches!(
                read_ok_response(&mut connector),
                Err(Error::UnexpectedResponse { response }) if response == "Good"
            ))
        }
    }
}
