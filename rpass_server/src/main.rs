pub mod storage;
mod request_dispatcher;
mod callbacks;
mod session;

use std::net::{TcpListener, TcpStream};
use std::io::{self, BufRead, BufReader, Write, Error, ErrorKind};
use std::borrow::Cow;
use std::sync::{Arc, RwLock};
use storage::Storage;
use request_dispatcher::{RequestDispatcher};
use session::Session;
#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), anyhow::Error> {
    let home_dir = dirs::home_dir().ok_or(
        Error::new(ErrorKind::NotFound, "Can't open home directory"))?;
    let path = home_dir.join(".rpass_storage");

    let storage = Arc::new(RwLock::new(Storage::from_path(path)?));
    let request_dispatcher = build_request_dispatcher(storage.clone());

    let listener = TcpListener::bind("127.0.0.1:3747")?;

    crossbeam_utils::thread::scope(|spawner| {
        for stream_res in listener.incoming() {
            let stream = match stream_res {
                Ok(connection) => connection,
                Err(_) => break
            };
            log_connection(&stream, true);

            let request_dispatcher_clone = request_dispatcher.clone();
            let storage_clone = storage.clone();
            spawner.spawn(move |_| handle_client(stream, storage_clone,
                request_dispatcher_clone));
        }
    }).unwrap();

    Ok(())
}

fn build_request_dispatcher(storage : Arc<RwLock<Storage>>)
        -> Arc<RwLock<RequestDispatcher>> {
    let request_dispatcher = Arc::new(RwLock::new(
        RequestDispatcher::default()));

    {
        let register_storage = storage.clone();
        let login_storage = storage.clone();
        let confirm_login_storage = storage.clone();
        let delete_me_storage = storage.clone();
        let new_record_storage = storage.clone();
        let show_record_storage = storage.clone();
        let list_records_storage = storage.clone();

        let mut dispatcher_write = request_dispatcher.write().unwrap();
        dispatcher_write
        .add_callback(Cow::from("register"), move |_, arg_iter| {
            callbacks::register(register_storage.clone(), arg_iter)
                .map_err(|err| err.into())
        })
        .add_callback(Cow::from("login"), move |session, arg_iter| {
            callbacks::login(login_storage.clone(), session, arg_iter)
                .map_err(|err| err.into())
        })
        .add_callback(Cow::from("confirm_login"), move |session, arg_iter| {
            callbacks::confirm_login(
                confirm_login_storage.clone(), session, arg_iter)
                .map_err(|err| err.into())
        })
        .add_callback(Cow::from("delete_me"), move |session, _| {
            callbacks::delete_me(delete_me_storage.clone(), session)
                .map_err(|err| err.into())
        })
        .add_callback(Cow::from("quit"), |session, _| {
            callbacks::quit(session).map_err(|err| err.into())
        })
        .add_callback(Cow::from("new_record"), move |session, arg_iter| {
            callbacks::new_record(new_record_storage.clone(), session, arg_iter)
                .map_err(|err| err.into())
        })
        .add_callback(Cow::from("show_record"), move |session, arg_iter| {
            callbacks::show_record(
                show_record_storage.clone(), session, arg_iter)
                .map_err(|err| err.into())
        })
        .add_callback(Cow::from("list_records"), move |session, _| {
            callbacks::list_records(list_records_storage.clone(), session)
                .map_err(|err| err.into())
        });
    }

    request_dispatcher
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

fn handle_client(mut stream: TcpStream,
        storage: Arc<RwLock<Storage>>,
        request_dispatcher: Arc<RwLock<RequestDispatcher>>)
        -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut session = Session::default();

    send_storage_key(&mut stream, storage)?;

    while !session.is_ended {
        let bytes = read_request_bytes(&mut reader)?;
        let mut request = match String::from_utf8(bytes) {
            Err(_) => {
                stream.write_all(
                    "Error: request should be in UTF-8 format\r\n".as_bytes())?;
                continue;
            },
            Ok(request) => request
        };
        request = request.trim().to_owned();
        println!("request = \"{}\"", request);

        let dispatcher_read = request_dispatcher.read().unwrap();
        let mut response = match dispatcher_read
                .dispatch(&mut session, &request) {
            Ok(response) => response,
            Err(err) => format!("Error: {}\r\n", err.to_string())
        };

        if !response.ends_with("\r\n") {
            response += "\r\n";
        }

        stream.write_all(response.as_bytes())?;
        request.clear();
    }

    log_connection(&stream, false);
    Ok(())
}

/// Sends storage pub key to the stream
fn send_storage_key(stream: &mut TcpStream, storage: Arc<RwLock<Storage>>)
        -> io::Result<()> {
    let storage_read = storage.read().unwrap();
    let pub_key = storage_read.get_pub_key();
    let message = pub_key.to_string() + "\r\n";
    stream.write_all(message.as_bytes())
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
