# FTP Server
This is an simple ftp server buit on rust. 
The server is running on locakhost and port 2020 and can be connected from any standard ftp client or custom made ftp client which contains the bellow mentioned command.

## Currently Implemented command
1. USER
2. PASS
3. SYST
4. PWD
5. LIST
6. RETR
7. QUIT

## Example Output:
It can be connected from any ftp client. The ftp server is running on localhost and on port 2020.
```bash
ftp 127.0.0.1 2020
```
Output:
```
Connected to 127.0.0.1.
220 The server is ready...
Name (127.0.0.1:imck037): imck037
331 Username is correct. Enter password for authentication.
Password:
230 Login Successfull.
Remote system type is UNIX.
Using binary mode to transfer files.
ftp> pwd
257 "/home/imck037/devs/ftp-server-rs"
ftp> quit
221 leaving the ftp server....
```