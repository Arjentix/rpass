mod storage;

use std::net::TcpListener;
use std::io::Write;
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
    let storage = Arc::new(RwLock::new(Storage::from_path("~/.rpass_storage")));

    let listener = TcpListener::bind("127.0.0.1:3747")?;

    crossbeam_utils::thread::scope(|spawner| {
        for stream_res in listener.incoming() {
            if let Err(_) = stream_res {
                break;
            }

            let stream = stream_res.unwrap();
            let addr = match stream.peer_addr() {
                Ok(peer_addr) => Cow::from(peer_addr.to_string()),
                Err(_) => Cow::from("unknown")
            };
            println!("Connected with {}", addr);

            let storage_clone = storage.clone();
            spawner.spawn(move |_| handle_client(stream, storage_clone));
        }
    }).unwrap();

    Ok(())
}
