use crate::key::Key;
use crate::{Error, Result};

use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::{tcp, TcpStream},
};

use std::str::FromStr;

#[cfg(test)]
use mockall::automock;

/// Connector that interacts with *rpass_db*
#[derive(Debug)]
pub struct Connector {
    stream: Box<TcpStream>,
    reader: BufReader<tcp::ReadHalf<'static>>,
    writer: tcp::WriteHalf<'static>,
    server_pub_key: Key,
}

/// End of transmission character
const EOT: u8 = 0x04;

#[cfg_attr(test, automock)]
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
    /// * `InvalidResponseEncoding` - if response isn't UTF-8 encoded
    pub async fn new(mut stream: Box<TcpStream>) -> Result<Self> {
        let stream_ptr: *mut TcpStream = &mut *stream;
        let (reader, writer) = unsafe { <*mut TcpStream>::as_mut(stream_ptr).unwrap().split() };
        let mut reader = BufReader::new(reader);
        let server_pub_key = Self::read_server_pub_key(&mut reader).await?;
        Ok(Connector {
            stream,
            reader,
            writer,
            server_pub_key,
        })
    }

    /// Receives response from server
    ///
    /// Returns response without EOT byte and "\r\n" ending if there is some
    ///
    /// # Errors
    ///
    /// * `Io` - if can't retrieve bytes from server
    /// * `InvalidResponseEncoding` - if response isn't UTF-8 encoded
    pub async fn recv_response(&mut self) -> Result<String> {
        read_response(&mut self.reader).await
    }

    /// Sends `request` to the server
    ///
    /// # Errors
    ///
    /// * `Io` - if can't send bytes to the server
    /// * `InvalidRequest` - if `request` contains EOT byte
    pub async fn send_request(&mut self, request: String) -> Result<()> {
        write_request(&mut self.writer, request).await
    }

    /// Reads server public key from `reader`
    ///
    /// # Errors
    ///
    /// * See [`read_response()`]
    /// * `InvalidKey` - if can't parse server key
    async fn read_server_pub_key<R: AsyncBufRead + Unpin + 'static>(reader: &mut R) -> Result<Key> {
        let key = read_response(reader).await?;
        Key::from_str(&key).map_err(|err| err.into())
    }

    /// Get a reference to the connector's server pub key.
    pub fn server_pub_key(&self) -> &Key {
        &self.server_pub_key
    }
}

/// Gracefully disconnects from server
impl Drop for Connector {
    fn drop(&mut self) {
        async {
            let _ = self.send_request(String::from("quit")).await;
        };
    }
}

/// Reads response from `reader`
///
/// Returns response without EOT byte and "\r\n" ending if there is some
///
/// # Errors
///
/// * `Io` - if can't read bytes from `reader`
/// * `InvalidResponseEncoding` - if response isn't UTF-8 encoded
async fn read_response<R: AsyncBufRead + Unpin>(mut reader: R) -> Result<String> {
    let mut buf = vec![];
    let size = reader.read_until(EOT, &mut buf).await?;
    if size == 0 {
        return Ok(String::new());
    }

    if *buf.last().unwrap() == EOT {
        buf.pop();
    }

    let response = String::from_utf8(buf)?;
    if let Some(stripped) = response.strip_suffix("\r\n") {
        return Ok(stripped.to_string());
    }

    Ok(response)
}

/// Writes `request` to `writer`
///
/// # Errors
///
/// * `Io` - if can't send bytes to `writer`
/// * `InvalidRequest` - if `request` contains EOT byte
async fn write_request<W: AsyncWrite + Unpin>(mut writer: W, request: String) -> Result<()> {
    writer
        .write_all(&make_request(request)?)
        .await
        .map_err(|err| err.into())
}

/// Takes raw `request` string, adds *"\r\n"* at the end if needed and
/// converts to bytes
fn make_request(mut request: String) -> Result<Vec<u8>> {
    if request.bytes().any(|byte| byte == EOT) {
        return Err(Error::InvalidRequest {
            mes: String::from("request should not contain EOT byte"),
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

    use std::io::Cursor;
    use std::task::Poll;
    use tokio::io::AsyncRead;

    /// Reader that fails to read
    struct TestReader;

    impl AsyncRead for TestReader {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "read error",
            )))
        }
    }

    #[tokio::test]
    async fn test_read_response_basic() {
        let mut reader = Cursor::new("response");
        assert_eq!(read_response(&mut reader).await.unwrap(), "response");
    }

    #[tokio::test]
    async fn test_read_response_empty() {
        let mut reader = Cursor::new("");
        assert_eq!(read_response(&mut reader).await.unwrap(), "");
    }

    #[tokio::test]
    async fn test_read_response_with_eot_at_the_end() {
        let mut response = String::from("response").into_bytes();
        response.push(EOT);

        let mut reader = Cursor::new(response);
        assert_eq!(read_response(&mut reader).await.unwrap(), "response");
    }

    #[tokio::test]
    async fn test_read_response_carriage_return() {
        let mut reader = Cursor::new("response\r\n");
        assert_eq!(read_response(&mut reader).await.unwrap(), "response");
    }

    #[tokio::test]
    async fn test_read_response_io_error() {
        let mut reader = BufReader::new(TestReader {});
        assert!(matches!(
            read_response(&mut reader).await,
            Err(Error::Io(_))
        ));
    }

    #[tokio::test]
    async fn test_read_response_invalid_response() {
        let mut reader = Cursor::new([0, 1, 128, EOT]);
        assert!(matches!(
            read_response(&mut reader).await,
            Err(Error::InvalidResponseEncoding(_))
        ));
    }

    #[tokio::test]
    async fn test_make_request_with_eot_at_the_end() {
        let mut bytes = "login".as_bytes().to_vec();
        bytes.push(EOT);
        bytes.extend_from_slice("user".as_bytes());

        let request = String::from_utf8(bytes).unwrap();
        assert!(matches!(
            make_request(request),
            Err(Error::InvalidRequest { .. })
        ))
    }

    #[tokio::test]
    async fn test_make_request_carriage_return() {
        let request = String::from("login user");
        let mut expected = (request.clone() + "\r\n").into_bytes();
        expected.push(EOT);

        assert_eq!(&make_request(request).unwrap(), &expected);
    }
}
