mod storage;
mod request_dispatcher;

use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write, Error, ErrorKind};
use std::borrow::Cow;
use std::sync::{Arc, RwLock};
use std::str::FromStr;
use storage::{Storage, Key};
use request_dispatcher::{RequestDispatcher};

fn main() -> std::io::Result<()> {
    let home_dir = dirs::home_dir().ok_or(
        Error::new(ErrorKind::NotFound, "Can't open home directory"))?;
    let path = home_dir.join(".rpass_storage");

    let storage = Arc::new(RwLock::new(Storage::from_path(path)));
    let request_dispatcher = build_request_dispatcher(storage);

    let listener = TcpListener::bind("127.0.0.1:3747")?;

    crossbeam_utils::thread::scope(|spawner| {
        for stream_res in listener.incoming() {
            let stream = match stream_res {
                Ok(connection) => connection,
                Err(_) => break
            };
            log_connection(&stream);

            let request_dispatcher_clone = request_dispatcher.clone();
            spawner.spawn(move |_| handle_client(stream,
                request_dispatcher_clone));
        }
    }).unwrap();

    Ok(())
}

fn build_request_dispatcher(storage : Arc<RwLock<Storage>>) -> Arc<RwLock<RequestDispatcher>> {
    let request_dispatcher = Arc::new(RwLock::new(RequestDispatcher::default()));

    let mut dispatcher_write = request_dispatcher.write().unwrap();
    {
        let storage_clone = storage.clone();
        dispatcher_write.add_callback("register".to_string(), move |arg_iter| {
            let username = match arg_iter.next() {
                Some(username) => username,
                None => return "Error: empty username".to_string()
            };
            println!("username = \"{}\"", username);
            let key_string = match arg_iter.next() {
                Some(key_string) => key_string,
                None => return "Error: empty key".to_string()
            };
            let key = match Key::from_str(key_string) {
                Ok(key) => key,
                Err(err) => return err.to_string()
            };

            let mut storage_write = storage_clone.write().unwrap();
            match storage_write.add_new_user(&username, &key) {
                Ok(()) => "Ok".to_string(),
                Err(err) => err.to_string()
            }
        });
    }

    drop(dispatcher_write);
    request_dispatcher
}

/// Logs stream peer address to the stdout
fn log_connection(stream: &TcpStream) {
    let addr = match stream.peer_addr() {
        Ok(peer_addr) => Cow::from(peer_addr.to_string()),
        Err(_) => Cow::from("unknown")
    };
    println!("Connected with {}", addr);
}

fn handle_client(mut stream: TcpStream,
        request_dispatcher: Arc<RwLock<RequestDispatcher>>)
        -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    if let Err(_) = reader.read_line(&mut request) {
        return stream.write_all(
            "Error: request should be in UTF-8 format".as_bytes());
    }
    request = request.trim().to_string();
    println!("request = \"{}\"", request);

    let dispatcher_read = request_dispatcher.read().unwrap();
    let response = dispatcher_read.dispatch(&request).unwrap_or(
        String::from("Error: invalid request"));

    stream.write_all(response.as_bytes())
}
