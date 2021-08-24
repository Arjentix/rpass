use std::net::TcpListener;
use std::io::Write;
use std::thread::spawn;
use std::borrow::Cow;

fn handle_client<T: Write>(mut stream: T) -> std::io::Result<()> {
    stream.write_all("Hello from rpass server!".as_bytes())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3747")?;
    let mut thread_handles = vec![];

    for stream_res in listener.incoming() {
        let stream = stream_res?;
        let addr = match stream.peer_addr() {
            Ok(peer_addr) => Cow::from(peer_addr.to_string()),
            Err(_) => Cow::from("unknown")
        };
        println!("Connected with {}", addr);

        thread_handles.push(
            spawn(move || handle_client(stream))
        );
    }

    for handle in thread_handles {
        handle.join().unwrap()?;
    }

    Ok(())
}
