use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

fn handle_client(mut stream: TcpStream) {
    stream.write(b"The server is ready...").unwrap();
    let mut buffer = [0; 1024];
    let message_lenth = stream.read(&mut buffer).unwrap();
    let response = String::from_utf8_lossy(&buffer[..message_lenth]);
    println!("{}", response);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:2020").expect("Failed to bind...");
    println!("The ftp server is running on port 2020");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                println!("Client connected...");
                thread::spawn(|| {
                    handle_client(s);
                });
            }
            Err(e) => {
                println!("Failed to connect to the client...{}", e);
            }
        }
    }
}
