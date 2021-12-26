use std::borrow::Cow;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

pub type Result<T> = io::Result<T>;

use crate::AsyncRequestDispatcher;
use crate::Session;

/// Server to handle clients requests
///
/// Allocates a new thread for every new connection
pub struct Server {
    listener: TcpListener,
    pub_key: String,
    dispatcher: AsyncRequestDispatcher,
}

impl Server {
    /// End of transmission character
    const EOT: u8 = 0x04;

    /// Creates new Server instance serving on `addr` with public key `pub_key`
    /// and `dispatcher` to handle clients
    pub fn new<A: ToSocketAddrs>(
        addr: A,
        pub_key: String,
        dispatcher: AsyncRequestDispatcher,
    ) -> Result<Self> {
        Ok(Server {
            listener: TcpListener::bind(addr)?,
            pub_key,
            dispatcher,
        })
    }

    /// Runs server
    pub fn run(&self) {
        crossbeam_utils::thread::scope(|spawner| {
            for stream_res in self.listener.incoming() {
                let stream = match stream_res {
                    Ok(connection) => connection,
                    Err(err) => {
                        println!("Failed to connect: {}", err);
                        break;
                    }
                };

                spawner.spawn(|_| self.handle_client(stream));
            }
        })
        .unwrap()
    }

    /// Handles client `stream`
    ///
    /// # Errors
    ///
    /// See [`handle_requests()`]
    fn handle_client(&self, mut stream: TcpStream) -> Result<()> {
        let addr = match stream.peer_addr() {
            Ok(peer_addr) => Cow::from(peer_addr.to_string()),
            Err(_) => Cow::from("unknown"),
        };
        log_connection(&addr, ConnectionStatus::Connected);

        let res = self.handle_requests(&mut stream);

        log_connection(&addr, ConnectionStatus::Disconnected);
        res
    }

    /// Handles requests from `stream` in cycle
    ///
    /// # Errors
    ///
    /// Any error caused by `stream` cloning, reading or writing
    fn handle_requests(&self, stream: &mut TcpStream) -> Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut session = Session::default();

        self.send_storage_key(stream)?;

        while !session.is_ended() {
            let bytes = Self::read_request_bytes(&mut reader)?;
            let request = String::from_utf8(bytes);

            let response = match request {
                Ok(request) => {
                    let request_str = request.trim();
                    println!("request = \"{}\"", request_str);
                    self.dispatch_request(&mut session, request_str)
                }
                Err(_) => "Error: request should be in UTF-8 format\r\n".to_owned(),
            };

            stream.write_all(&Self::response_to_bytes(response))?;
        }

        Ok(())
    }

    /// Sends storage pub key to the `stream`
    ///
    /// # Errors
    ///
    /// See [`TcpStream::write_all()`]
    fn send_storage_key(&self, stream: &mut TcpStream) -> Result<()> {
        let bytes = Self::response_to_bytes(self.pub_key.clone() + "\r\n");
        stream.write_all(&bytes)
    }

    /// Dispatches `request` with `session` using `self.dispatcher`
    ///
    /// Returns response with "\r\n" at the end
    fn dispatch_request(&self, session: &mut Session, request: &str) -> String {
        let dispatcher_read = self.dispatcher.read().unwrap();
        let mut response = match dispatcher_read.dispatch(session, request) {
            Ok(response) => response,
            Err(err) => format!("Error: {}\r\n", err),
        };

        if !response.ends_with("\r\n") {
            response += "\r\n";
        }
        response
    }

    /// Reads bytes from `reader` until EOT byte is captured.
    /// Returns bytes without EOT byte
    fn read_request_bytes<R: BufRead>(mut reader: R) -> Result<Vec<u8>> {
        let mut buf = vec![];
        reader.read_until(Self::EOT, &mut buf)?;
        buf.pop();

        Ok(buf)
    }

    /// Converts `response` to bytes with EOT byte at the end
    fn response_to_bytes(mut response: String) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(response.len() + 1);
        unsafe {
            bytes.append(response.as_mut_vec());
        }
        bytes.push(Self::EOT);
        bytes
    }
}

/// Status of connection with client
///
/// Used to improve log_connection() usage code readability
enum ConnectionStatus {
    Connected,
    Disconnected,
}

/// Logs status of connection with `peer_addr` to the stdout.
/// If `connection` is *ConnectionStatus::Connected* prints info about
/// successful connection. Else prints info about disconnection
fn log_connection(peer_addr: &str, connection: ConnectionStatus) {
    match connection {
        ConnectionStatus::Connected => println!("Connected with {}", peer_addr),
        ConnectionStatus::Disconnected => println!("Connection with {} closed", peer_addr),
    }
}
