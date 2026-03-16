# FTP Server
This is an RFC 959 standard FTP server buit on rust. 
This server is consists of all the fetures of an FTP server based on RFC 959 Standard.

- Both Control Connection and Data Connection is featured.
- User Authentication is implemented using inbuilt session.
- The server can be connected and used by any standard FTP client (Fileailla, Gnu inetutils ftp client)
- It can be used by any custom made client with raw ftp commands and it has inbuilt user authentication system that allow the server to work properly using user session.


## Currently Implemented command
Right now it support almost every command that an ftp server need to operate.
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
4. STOR

### Data Connection
1. PASV
2. PORT

PORT command is removed and redirected to use pasv as most firewall dont allow the connection.

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