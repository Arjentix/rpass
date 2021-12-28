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

    const ERROR_PREFIX: &str = "Error: ";
    if let Some(stripped) = response.strip_prefix(ERROR_PREFIX) {
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
