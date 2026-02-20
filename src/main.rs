use std::env;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    stream.write(b"The server is ready...\n").unwrap();

    loop {
        let mut command = String::new();
        if reader.read_line(&mut command).is_err() {
            break;
        }
        let command = command.trim();
        println!("Command: {command}");

        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0].to_lowercase().as_str() {
            "user" => {
                stream
                    .write_all(b"Username is correct. Enter password for authentication.\r\n")
                    .unwrap();
            }
            "pass" => {
                stream.write_all(b"230 Login Successfull.\r\n").unwrap();
            }
            "pwd" => {
                let dir = env::current_dir().unwrap();
                let dir_response = format!("257 \"{}\"\r\n", dir.display());
                stream.write_all(dir_response.as_bytes()).unwrap();
            }
            "list" => {
                stream
                    .write_all(b"150 Listing directory and files..")
                    .unwrap();
                let dirs = fs::read_dir(".").unwrap();
                for dir in dirs {
                    let dir = dir.unwrap();
                    let file_name = dir.file_name();
                    let file_name = file_name.to_string_lossy();

                    let dirs_response_line = format!("{}", file_name);
                    stream.write_all(dirs_response_line.as_bytes()).unwrap();
                }
            }
            _ => {
                stream.write_all(b"Command is not specified..").unwrap();
            }
        }
    }
    println!("Client disconnect..");
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
