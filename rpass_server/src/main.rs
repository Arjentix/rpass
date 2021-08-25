mod storage;

use std::net::TcpListener;
use std::io::Write;
use std::borrow::Cow;
use storage::Storage;

fn handle_client<T: Write>(mut stream: T, _storage: &Storage) -> std::io::Result<()> {
    stream.write_all("Hello from rpass server!".as_bytes())
}

fn main() -> std::io::Result<()> {
    let storage = Storage::from_path("~/.rpass_storage");
    let storage_ref = &storage;

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

            spawner.spawn(move |_| handle_client(stream, storage_ref));
        }
    }).unwrap();

    Ok(())
}
