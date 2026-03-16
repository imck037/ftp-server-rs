use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time;

#[derive(Clone)]
struct UserInfo {
    username: String,
    password: String,
}

struct Connection {
    data_connection: Option<TcpStream>,
}

#[derive(Clone, Copy)]
struct ServerStat {
    starttime: time::Instant,
    connected_user: u16,
}

fn handle_client(stream: &mut TcpStream, stat: ServerStat, userinfo: UserInfo) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut conn = Connection {
        data_connection: None,
    };
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
                handle_username(stream, parts, userinfo.clone());
            }
            "PASS" => {
                handle_pass(stream, parts, userinfo.clone());
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
                conn.data_connection = Some(new_stream);
            }
            "PORT" => {
                stream
                    .write_all(b"200 Port command not supported please use PASV\r\n")
                    .unwrap();
                continue;
            }
            "PWD" => {
                let dir = env::current_dir().unwrap();
                let dir_response = format!("257 \"{}\"\r\n", dir.display());
                stream.write_all(dir_response.as_bytes()).unwrap();
            }
            "LIST" => {
                handle_list(stream, &mut conn);
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
                handle_retr(stream, &mut conn, parts);
            }
            "STOR" => {
                handle_store(stream, &mut conn, parts);
            }
            "QUIT" => {
                stream
                    .write_all(b"221 leaving the ftp server....\r\n")
                    .unwrap();
                break;
            }
            _ => {
                stream
                    .write_all(b"502 Command is not specified..\r\n")
                    .unwrap();
            }
        }
    }
    println!("Client disconnect..");
}

fn handle_username(stream: &mut TcpStream, parts: Vec<&str>, userinfo: UserInfo) {
    if userinfo.username == parts[1].to_string().trim() {
        stream
            .write_all(b"331 Username is Ok. Password needed.\r\n")
            .unwrap();
    } else {
        println!("{:?}", parts[1]);
        stream.write_all(b"530 Wrong username.\r\n").unwrap();
    }
}

fn handle_pass(stream: &mut TcpStream, parts: Vec<&str>, userinfo: UserInfo) {
    if userinfo.password == parts[1].to_string().trim() {
        stream.write_all(b"230 Login Successfull.\r\n").unwrap();
    } else {
        stream.write_all(b"550 Wrong Password.\r\n").unwrap();
    }
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

fn handle_store(stream: &mut TcpStream, conn: &mut Connection, parts: Vec<&str>) {
    if parts.len() < 2 {
        stream
            .write_all(b"226 File is not specified...\r\n")
            .unwrap();
        return;
    }
    if let Some(datastream) = &mut conn.data_connection {
        stream
            .write_all(b"150 Opening Data Connection.\r\n")
            .unwrap();
        if let Ok(mut file_buffer) = fs::File::create(parts[1]) {
            let mut buffer = String::new();
            datastream.read_to_string(&mut buffer).unwrap();
            file_buffer.write_all(buffer.as_bytes()).unwrap();
            stream.write_all(b"226 Transfer complete.\r\n").unwrap();
            conn.data_connection = None;
        } else {
            stream.write_all(b"550 Cannot create file.\r\n").unwrap();
        }
    } else {
        stream
            .write_all(b"425 Cannot Open Data Connection Please Use PASV.\r\n")
            .unwrap();
    }
}

fn handle_retr(stream: &mut TcpStream, conn: &mut Connection, parts: Vec<&str>) {
    if parts.len() < 2 {
        stream
            .write_all(b"226 File is not specified...\r\n")
            .unwrap();
        return;
    }
    let filename = &parts[1];

    if let Some(datastream) = &mut conn.data_connection {
        match fs::read(filename) {
            Ok(data) => {
                stream.write_all(b"150 opening file\r\n").unwrap();
                datastream.write_all(&data).unwrap();
                stream.write_all(b"226 transfer complete\r\n").unwrap();
            }
            Err(_) => {
                stream.write_all(b"550 file not founc...\r\n").unwrap();
            }
        }
        conn.data_connection = None;
    } else {
        stream
            .write_all(b"425 Please use pasv mode to receive file\r\n")
            .unwrap();
    }
}

fn handle_list(stream: &mut TcpStream, conn: &mut Connection) {
    if let Some(data_stream) = &mut conn.data_connection {
        stream
            .write_all(b"150 Listing directory and files..\r\n")
            .unwrap();
        let dirs = fs::read_dir(".").unwrap();
        for dir in dirs {
            let dir = dir.unwrap();
            let file_name = dir.file_name();
            let file_name = file_name.to_string_lossy();

            let dirs_response_line = format!("{}\r\n", file_name);
            data_stream
                .write_all(dirs_response_line.as_bytes())
                .unwrap();
        }
        stream
            .write_all(b"226 Directory fetched successfully.\r\n")
            .unwrap();
        conn.data_connection = None;
    } else {
        stream
            .write_all(b"425 Data connection is not established consider using PASV\r\n")
            .unwrap();
    }
}

fn create_users() -> UserInfo {
    let mut username = String::new();
    let mut password = String::new();

    println!("Create a new user....");
    print!("enter the username: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut username).unwrap();
    print!("enter the password: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut password).unwrap();

    println!("Server Configured.....");
    UserInfo {
        username: username.trim().to_string(),
        password: password.trim().to_string(),
    }
}

fn main() {
    println!("Configuring the server...");
    let config = create_users();

    let listener = TcpListener::bind("127.0.0.1:2020").expect("Failed to bind...");
    println!("The ftp server is running on port 2020");
    let mut stat = ServerStat {
        starttime: time::Instant::now(),
        connected_user: 0,
    };
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                println!("Client connected...");
                let config = config.clone();
                stat.connected_user += 1;
                thread::spawn(move || {
                    handle_client(&mut s, stat, config);
                });
            }
            Err(e) => {
                println!("Failed to connect to the client...{}", e);
            }
        }
    }
}
