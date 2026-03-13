use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::Ipv4Addr;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time;

#[derive(Clone, Copy)]
struct ServerStat {
    starttime: time::Instant,
    connected_user: u16,
}

fn handle_client(mut stream: TcpStream, stat: ServerStat) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    stream.write_all(b"220 The server is ready...\n").unwrap();

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
            "SYST" => {
                stream.write_all(b"215 unix type: l8\r\n").unwrap();
            }
            "STAT" => {
                handle_stat(&stream, &stat);
            }
            "NOOP" => {
                stream.write_all(b"200 Command successful.\r\n").unwrap();
            }
            "PASV" => {
                let new_stream = handle_pasv(&stream);
                println!("Client connected to {}", new_stream.peer_addr().unwrap());
            }
            "PORT" => {
                if parts.len() < 2 {
                    stream
                        .write_all(b"226 File is not specified...\r\n")
                        .unwrap();
                    continue;
                }
                let ip_port: Vec<&str> = parts[1].split(",").collect();
                let ip = Ipv4Addr::new(
                    ip_port[0].parse().unwrap(),
                    ip_port[1].parse().unwrap(),
                    ip_port[2].parse().unwrap(),
                    ip_port[3].parse().unwrap(),
                );
                let port1: u16 = ip_port[4].parse().unwrap();
                let port2: u16 = ip_port[5].parse().unwrap();
                let port = port1 * 255 + port2;
                let new_addr = format!("{}:{}", ip, port);
                let new_stream =
                    TcpStream::connect(new_addr).expect("Problem connecting the new port");
                println!(
                    "Client connected to {}",
                    new_stream.peer_addr().expect("cannot get the ip addr")
                );
                stream
                    .write_all(b"200 Port command succesfull\r\n")
                    .unwrap();
            }
            "PWD" => {
                let dir = env::current_dir().unwrap();
                let dir_response = format!("257 \"{}\"\r\n", dir.display());
                stream.write_all(dir_response.as_bytes()).unwrap();
            }
            "LIST" => {
                stream
                    .write_all(b"150 Listing directory and files..\r\n")
                    .unwrap();
                let dirs = fs::read_dir(".").unwrap();
                for dir in dirs {
                    let dir = dir.unwrap();
                    let file_name = dir.file_name();
                    let file_name = file_name.to_string_lossy();

                    let dirs_response_line = format!("{}  ", file_name);
                    stream.write_all(dirs_response_line.as_bytes()).unwrap();
                }
                stream
                    .write_all(b"226 Directory fetched successfully.\r\n")
                    .unwrap();
            }
            "MKD" => {
                if parts.len() < 2 {
                    stream
                        .write_all(b"226 Directory name is needed\r\n")
                        .unwrap();
                }
                let dir = fs::create_dir(parts[1]);
                match dir {
                    Ok(_) => stream
                        .write_all(b"257 Directory created successfully.\r\n")
                        .unwrap(),
                    Err(_) => stream
                        .write_all(b"500 Cannot create directory.\r\n")
                        .unwrap(),
                }
            }
            "RMD" => {
                if parts.len() < 2 {
                    stream
                        .write_all(b"226 Directory name is needed\r\n")
                        .unwrap();
                }
                let dir = fs::remove_dir_all(parts[1]);
                match dir {
                    Ok(_) => stream
                        .write_all(b"250 Directory removed successfully.\r\n")
                        .unwrap(),
                    Err(_) => stream
                        .write_all(b"500 Cannot remove directory.\r\n")
                        .unwrap(),
                }
            }
            "DELE" => {
                if parts.len() < 2 {
                    stream.write_all(b"226 File name is needed\r\n").unwrap();
                }
                let dir = fs::remove_file(parts[1]);
                match dir {
                    Ok(_) => stream
                        .write_all(b"250 file removed successfully.\r\n")
                        .unwrap(),
                    Err(_) => stream
                        .write_all(b"550 Cannot remove file or permission denied.\r\n")
                        .unwrap(),
                }
            }
            "SIZE" => {
                if parts.len() < 2 {
                    stream.write_all(b"226 File name is needed\r\n").unwrap();
                }
                let metadata = fs::metadata(parts[1]);
                match metadata {
                    Ok(data) => {
                        let file_size = data.len();
                        let response = format!("213 {}\r\n", file_size);
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                    Err(_) => stream.write_all(b"550 File not found.\r\n").unwrap(),
                }
            }
            "CWD" => {
                if parts.len() < 2 {
                    stream.write_all(b"Directory name is needed\r\n").unwrap();
                }
                match env::set_current_dir(parts[1]) {
                    Ok(_) => stream
                        .write_all(b"250 Directory changed successfully.\r\n")
                        .unwrap(),
                    Err(_) => stream.write_all(b"550 Directory not found.\r\n").unwrap(),
                }
            }
            "CDUP" => match env::set_current_dir("..") {
                Ok(_) => stream
                    .write_all(b"250 Directory changed to parent successfully.\r\n")
                    .unwrap(),
                Err(_) => stream.write_all(b"550 Directory not found.\r\n").unwrap(),
            },
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

fn handle_pasv(mut stream: &TcpStream) -> TcpStream {
    let data_socket = TcpListener::bind("0.0.0.0:0").expect("Unable open new port.");

    let port = data_socket.local_addr().unwrap().port();

    let p1 = port / 256;
    let p2 = port % 256;
    let ip = "127,0,0,1";

    let response = format!("227 Entering Passive Mode ({ip},{p1},{p2})\r\n");
    stream.write_all(response.as_bytes()).unwrap();
    let (new_stream, _) = data_socket.accept().unwrap();
    new_stream
}

fn handle_stat(mut stream: &TcpStream, stat: &ServerStat) {
    let uptime = stat.starttime.elapsed().as_secs();
    stream.write_all(b"211-Ftp server status").unwrap();
    let response = format!("Uptime: {}\r\n", uptime);
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(b"211 End of status.\r\n").unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:2020").expect("Failed to bind...");
    println!("The ftp server is running on port 2020");
    let mut stat = ServerStat {
        starttime: time::Instant::now(),
        connected_user: 0,
    };
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                println!("Client connected...");
                stat.connected_user += 1;
                thread::spawn(move || {
                    handle_client(s, stat);
                });
            }
            Err(e) => {
                println!("Failed to connect to the client...{}", e);
            }
        }
    }
}
