use crate::{Result, Error};
use crate::key::Key;
use std::net::TcpStream;
use std::io::{BufRead, BufReader, Write};
use std::str::FromStr;

/// Connector that interacts with *rpass_server*
#[derive(Debug)]
pub struct Connector {
    stream: TcpStream,
    buf_stream_reader: BufReader<TcpStream>,
    pub_key: Key,
    sec_key: Key,
    server_pub_key: Key
}

/// End of transmission character
const EOT: u8 = 0x04;

impl Connector {
    /// Creates new Connector
    ///
    /// Reads server pub key from `stream`
    ///
    /// # Errors
    ///
    /// * `Io` - if can't clone `stream` or some error during writing/reading
    /// bytes to/from server
    /// * `InvalidKey` - if can't parse server key
    /// * `InvalidResponse` - if response isn't UTF-8 encoded
    pub fn new(stream: TcpStream, pub_key: Key, sec_key: Key) -> Result<Self> {
        let mut buf_stream_reader = BufReader::new(stream.try_clone()?);
        let server_pub_key = Self::read_server_pub_key(&mut buf_stream_reader)?;
        Ok(Connector {
            stream,
            buf_stream_reader,
            pub_key,
            sec_key,
            server_pub_key
        })
    }

    /// Receives response from server
    ///
    /// Returns response without EOT byte and "\r\n" ending if there is some
    ///
    /// # Errors
    ///
    /// * `Io` - if can't retrieve bytes from server
    /// * `InvalidResponse` - if response isn't UTF-8 encoded
    pub fn recv_response(&mut self) -> Result<String> {
        read_response(&mut self.buf_stream_reader)
    }

    /// Sends `request` to the server
    ///
    /// # Errors
    ///
    /// * `Io` - if can't send bytes to the server
    /// * `InvalidRequest` - if `request` contains EOT byte
    pub fn send_request(&mut self, request: String) -> Result<()> {
        write_request(&mut self.stream, request)
    }

    /// Reads server public key from `reader`
    ///
    /// # Errors
    ///
    /// * See [`read_response()`]
    /// * `InvalidKey` - if can't parse server key
    fn read_server_pub_key<R: BufRead>(reader: &mut R) -> Result<Key> {
        let key = read_response(reader)?;
        Key::from_str(&key).map_err(|err| err.into())
    }

    /// Get a reference to the connector's pub key.
    pub fn pub_key(&self) -> &Key {
        &self.pub_key
    }

    /// Get a reference to the connector's sec key.
    pub fn sec_key(&self) -> &Key {
        &self.sec_key
    }

    /// Get a reference to the connector's server pub key.
    pub fn server_pub_key(&self) -> &Key {
        &self.server_pub_key
    }
}

/// Gracefully disconnects from server
impl Drop for Connector {
    fn drop(&mut self) {
        let _ = self.send_request(String::from("quit"));
    }
}

/// Reads response from `reader`
///
/// Returns response without EOT byte and "\r\n" ending if there is some
///
/// # Errors
///
/// * `Io` - if can't read bytes from `reader`
/// * `InvalidResponse` - if response isn't UTF-8 encoded
fn read_response<R: BufRead>(reader: &mut R) -> Result<String> {
    let mut buf = vec![];
    let size = reader.read_until(EOT, &mut buf)?;
    if size == 0 {
        return Ok(String::new());
    }

    if *buf.last().unwrap() == EOT {
        buf.pop();
    }

    let mut response = String::from_utf8(buf)?;
    if response.ends_with("\r\n") {
        for _ in response.drain(response.len() - 2..) {}
    }

    Ok(response)
}

/// Writes `request` to `writer`
///
/// # Errors
///
/// * `Io` - if can't send bytes to `writer`
/// * `InvalidRequest` - if `request` contains EOT byte
fn write_request<W: Write>(writer: &mut W, request: String) -> Result<()> {
    writer.write_all(&make_request(request)?).map_err(|err| err.into())
}

/// Takes raw `request` string, adds *"\r\n"* at the end if needed and
/// converts to bytes
fn make_request(mut request: String) -> Result<Vec<u8>> {
    if request.bytes().any(|byte| byte == EOT) {
        return Err(Error::InvalidRequest {
            mes: String::from("request should not contain EOT byte")
        });
    }

    if !request.ends_with("\r\n") {
        request += "\r\n";
    }

    let mut bytes = request.into_bytes();
    bytes.push(EOT);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Cursor};

    /// Reader that fails to read
    struct TestReader;

    impl Read for TestReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "read error"))
        }
    }

    #[test]
    fn test_read_response_basic() {
        let mut reader = Cursor::new("response");
        assert_eq!(read_response(&mut reader).unwrap(), "response");
    }

    #[test]
    fn test_read_response_empty() {
        let mut reader = Cursor::new("");
        assert_eq!(read_response(&mut reader).unwrap(), "");
    }

    #[test]
    fn test_read_response_with_eot_at_the_end() {
        let mut response = String::from("response").into_bytes();
        response.push(EOT);

        let mut reader = Cursor::new(response);
        assert_eq!(read_response(&mut reader).unwrap(), "response");
    }

    #[test]
    fn test_read_response_carriage_return() {
        let mut reader = Cursor::new("response\r\n");
        assert_eq!(read_response(&mut reader).unwrap(), "response");
    }

    #[test]
    fn test_read_response_io_error() {
        let mut reader = BufReader::new(TestReader{});
        assert!(matches!(read_response(&mut reader),
            Err(Error::Io(_))));
    }

    #[test]
    fn test_read_response_invalid_response() {
        let mut reader = Cursor::new([0, 1, 128, EOT]);
        assert!(matches!(read_response(&mut reader),
            Err(Error::InvalidResponse(_))));
    }

    #[test]
    fn test_make_request_with_eot_at_the_end() {
        let mut bytes = "login".as_bytes().to_vec();
        bytes.push(EOT);
        bytes.extend_from_slice("user".as_bytes());

        let request = String::from_utf8(bytes).unwrap();
        assert!(matches!(make_request(request), Err(Error::InvalidRequest{..})))
    }


    #[test]
    fn test_make_request_carriage_return() {
        let request = String::from("login user");
        let mut expected = (request.clone() + "\r\n").into_bytes();
        expected.push(EOT);

        assert_eq!(&make_request(request).unwrap(), &expected);
    }
}
