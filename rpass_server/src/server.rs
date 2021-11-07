pub type Result<T> = std::result::Result<T, std::io::Error>;

use std::borrow::Cow;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::io::{self, BufRead, BufReader, Write};

use crate::AsyncStorage;
use crate::AsyncRequestDispatcher;
use crate::Session;

/// Server to handle clients requests
///
/// Allocates a new thread for every new connection
pub struct Server {
    listener: TcpListener,
    storage: AsyncStorage,
    dispatcher: AsyncRequestDispatcher
}

impl Server {

    /// Creates new Server instance serving on `addr`, using `storage` and `dispatcher` to handle
    /// clients
    pub fn new<A: ToSocketAddrs>(addr: A, storage: AsyncStorage,
            dispatcher: AsyncRequestDispatcher) -> Result<Self> {
        Ok(Server {
            listener: TcpListener::bind(addr)?,
            storage,
            dispatcher
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
                log_connection(&stream, true);

                spawner.spawn(|_| self.handle_client(stream));
            }
        }).unwrap()
    }

    /// Handles client `stream`
    fn handle_client(&self, mut stream: TcpStream) -> Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut session = Session::default();

        self.send_storage_key(&mut stream)?;

        while !session.is_ended() {
            let bytes = read_request_bytes(&mut reader)?;
            let request = match String::from_utf8(bytes) {
                Err(_) => {
                    stream.write_all(
                        "Error: request should be in UTF-8 format\r\n"
                            .as_bytes())?;
                    continue;
                },
                Ok(request) => request.trim().to_owned()
            };
            println!("request = \"{}\"", request);

            let dispatcher_read = self.dispatcher.read().unwrap();
            let mut response = match dispatcher_read
                    .dispatch(&mut session, &request) {
                Ok(response) => response,
                Err(err) => format!("Error: {}\r\n", err.to_string())
            };

            if !response.ends_with("\r\n") {
                response += "\r\n";
            }

            stream.write_all(response.as_bytes())?;
        }

        log_connection(&stream, false);
        Ok(())
    }

    /// Sends storage pub key to the stream
    fn send_storage_key(&self, stream: &mut TcpStream)
            -> io::Result<()> {
        let storage_read = self.storage.read().unwrap();
        let pub_key = storage_read.get_pub_key();
        let message = pub_key.to_string() + "\r\n";
        stream.write_all(message.as_bytes())
    }
}

/// Logs `stream` peer address to the stdout. If `connected` prints info about
/// successful connection. Else prints info about disconnection
fn log_connection(stream: &TcpStream, connected: bool) {
    let addr = match stream.peer_addr() {
        Ok(peer_addr) => Cow::from(peer_addr.to_string()),
        Err(_) => Cow::from("unknown")
    };
    if connected {
        println!("Connected with {}", addr);
    } else {
        println!("Connection with {} closed", addr);
    }
}

/// Reads bytes from `reader` until EOT byte is captured.
/// Returns bytes without EOT byte
fn read_request_bytes(reader: &mut BufReader<TcpStream>)
        -> io::Result<Vec<u8>> {
    const EOT: u8 = 0x04;
    let mut buf = vec![];
    reader.read_until(EOT, &mut buf)?;
    buf.pop();

    Ok(buf)
}
