mod storage;

use std::net::{TcpListener, TcpStream};
use std::io::{Write, Error, ErrorKind};
use std::borrow::Cow;
use std::sync::{Arc, RwLock};
use storage::Storage;

fn handle_client<T: Write>(mut stream: T, storage: Arc<RwLock<Storage>>) -> std::io::Result<()> {
    stream.write_all("Hello from rpass server!".as_bytes())?;
    let mut storage_write = storage.write().unwrap();
    storage_write.edit();
    Ok(())
}

fn main() -> std::io::Result<()> {
    let home_dir = dirs::home_dir().ok_or(
        Error::new(ErrorKind::NotFound, "Can't open home directory"))?;
    let path = home_dir.join(".rpass_storage");
    let storage = Arc::new(RwLock::new(Storage::from_path(path)));

    let listener = TcpListener::bind("127.0.0.1:3747")?;

    crossbeam_utils::thread::scope(|spawner| {
        for stream_res in listener.incoming() {
            if let Err(_) = stream_res {
                break;
            }

            let stream = stream_res.unwrap();
            log_connection(&stream);

            let storage_clone = storage.clone();
            spawner.spawn(move |_| handle_client(stream, storage_clone));
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
