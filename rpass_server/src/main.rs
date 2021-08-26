mod storage;
mod request_dispatcher;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, Error, ErrorKind};
use std::borrow::Cow;
use std::sync::{Arc, RwLock};
use storage::Storage;
use request_dispatcher::{RequestDispatcher};

fn handle_client<S: Write + Read>(mut stream: S, _storage: Arc<RwLock<Storage>>,
        request_dispatcher: Arc<RwLock<RequestDispatcher>>)
        -> std::io::Result<()> {
    const BUF_SIZE: usize = 512;
    let mut raw_buf = vec![0u8; BUF_SIZE];
    stream.read(&mut raw_buf)?;
    let request = match String::from_utf8(raw_buf) {
        Ok(str) => str,
        Err(_) => return stream.write_all(
            "Error: request should be in UTF-8 encoded form".as_bytes())
    };

    let mut request_dispatcher_write = request_dispatcher.write().unwrap();
    let response = request_dispatcher_write.dispatch(&request).unwrap_or(
        String::from("Error: invalid request"));

    stream.write_all(response.as_bytes())
}

fn main() -> std::io::Result<()> {
    let home_dir = dirs::home_dir().ok_or(
        Error::new(ErrorKind::NotFound, "Can't open home directory"))?;
    let path = home_dir.join(".rpass_storage");
    let storage = Arc::new(RwLock::new(Storage::from_path(path)));
    let request_dispatcher = Arc::new(RwLock::new(RequestDispatcher::default()));

    let listener = TcpListener::bind("127.0.0.1:3747")?;

    crossbeam_utils::thread::scope(|spawner| {
        for stream_res in listener.incoming() {
            let stream = match stream_res {
                Ok(connection) => connection,
                Err(_) => break
            };
            log_connection(&stream);

            let storage_clone = storage.clone();
            let request_dispatcher_clone = request_dispatcher.clone();
            spawner.spawn(move |_| handle_client(stream, 
                storage_clone, request_dispatcher_clone));
        }
    }).unwrap();

    Ok(())
}

/// Logs stream peer address to the stdout
fn log_connection(stream: &TcpStream) {
    let addr = match stream.peer_addr() {
        Ok(peer_addr) => Cow::from(peer_addr.to_string()),
        Err(_) => Cow::from("unknown")
    };
    println!("Connected with {}", addr);
}
