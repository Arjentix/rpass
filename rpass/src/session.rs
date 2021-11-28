pub use crate::{Error, Result};

use std::net::{TcpStream, ToSocketAddrs};
use std::io::{self, BufReader, BufRead, Write};
use std::str::FromStr;
use crate::key::Key;
use enum_as_inner::EnumAsInner;

/// Enum representing user session
#[derive(EnumAsInner, Debug)]
pub enum Session {
    Unauthorized(Unauthorized),
    Authorized(Authorized)
}

/// Common data for Unauthorized and Authorized structs
#[derive(Debug)]
struct CommonData {
    stream: TcpStream,
    pub_key: Key,
    sec_key: Key,
    server_pub_key: Key
}

#[derive(Debug)]
pub struct Unauthorized {
    common_data: CommonData
}

#[derive(Debug)]
pub struct Authorized {
    common_data: CommonData
}

/// End of transmission character
const EOT: u8 = 0x04;

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
        let mut buf_stream = BufReader::new(stream.try_clone()?);
        let server_pub_key = Self::read_server_pub_key(&mut buf_stream)?;

        let common_data = CommonData {
            stream, pub_key, sec_key, server_pub_key
        };
        Ok(Session::Unauthorized(Unauthorized{common_data}))
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
}

/// Gracefully disconnects from server
///
/// Implemented for CommonData to avoid same code for `Unauthorized` and
/// `Authorized` structs
impl Drop for CommonData {
    fn drop(&mut self) {
        let _ = send_request(&mut self.stream, String::from("quit"));
    }
}

impl Unauthorized {
    pub fn login(self, _username: &str) -> Result<Authorized> {
        // TODO impl login
        Err(Error::Login(self))
    }
}

impl Authorized {
    // TODO impl authorized functions
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
    reader.read_until(EOT, &mut buf)?;
    buf.pop();

    let mut response = String::from_utf8(buf)?;
    if response.ends_with("\r\n") {
        response.remove(response.len() - 1);
        response.remove(response.len() - 1);
    }

    Ok(response)
}

fn send_request<W: Write>(writer: &mut W, request: String) -> io::Result<()> {
    writer.write_all(&make_request(request))
}

/// Takes raw `request` string, adds *"\r\n"* at the end if needed and
/// converts to bytes
fn make_request(mut request: String) -> Vec<u8> {
    if !request.ends_with("\r\n") {
        request += "\r\n";
    }

    let mut bytes = Vec::with_capacity(request.len() + 1);
    unsafe {
        bytes.append(request.as_mut_vec());
    }
    bytes.push(EOT);
    bytes
}
