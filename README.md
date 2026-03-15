# FTP Server
This is an RFC 959 standard FTP server buit on rust. 
This server is consists of all the fetures of an FTP server based on RFC 959 Standard. It's send the response using both the control connection and data connection like an standard server.
The server is currently written for running on localhost and port 2020 and can be connected from any standard ftp client or custom made ftp client which contains the bellow mentioned command.

## Currently Implemented command
### Connection and Authentication
1. USER
2. PASS
3. QUIT

### Directrory navigation
1. PWD
2. CWD
3. RMD
4. MKD
9. CDUP

### File managment
1. DELE
2. LIST
3. RETR

### Data Connection
1. PASV
2. PORT
##### PORT command is removed and redirected to use pasv as most firewall dont allow the connection.

### Status and Control
1. SYST
2. NOOP
3. STAT

## Example Output:
To start the server, use cargo
````
cargo run
````
Or by building the executable file through cargo
````
cargo build --release
./target/release/ftp-server-rs
````
It can be connected from any ftp client. The ftp server is running on localhost and on port 2020.
```bash
ftp 127.0.0.1 2020
```