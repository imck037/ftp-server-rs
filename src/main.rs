use std::env;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    stream.write(b"220 The server is ready...\n").unwrap();

    loop {
        let mut command = String::new();
        if reader.read_line(&mut command).is_err() {
            break;
        }
        let command = command.trim();

        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        println!("Command: {command}");

        match parts[0].to_uppercase().as_str() {
            "USER" => {
                stream
                    .write_all(b"331 Username is correct. Enter password for authentication.\r\n")
                    .unwrap();
            }
            "PASS" => {
                stream.write_all(b"230 Login Successfull.\r\n").unwrap();
            }
            "PWD" => {
                let dir = env::current_dir().unwrap();
                let dir_response = format!("257 \"{}\"\r\n", dir.display());
                stream.write_all(dir_response.as_bytes()).unwrap();
            }
            "LIST" => {
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
                stream
                    .write_all(b"226 Directory fetched successfully.\r\n")
                    .unwrap();
            }
            "RETR" => {
                if parts.len() < 2 {
                    stream
                        .write_all(b"226 File is not specified...\r\n")
                        .unwrap();
                    continue;
                }
                let filename = parts[1];

                match fs::read(filename) {
                    Ok(data) => {
                        stream.write_all(b"150 opening file\r\n").unwrap();
                        stream.write_all(&data).unwrap();
                        stream.write_all(b"\r\n226 transfer complete\r\n").unwrap();
                    }
                    Err(_) => {
                        stream.write_all(b"550 file not founc...\r\n").unwrap();
                    }
                }
            }
            "QUIT" => {
                stream
                    .write_all(b"221 leaving the ftp server....\r\n")
                    .unwrap();
                break;
            }
            _ => {
                stream.write_all(b"502 Command is not specified..").unwrap();
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
