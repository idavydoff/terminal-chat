# Terminal Chat
Chat service written in Rust. Includes server and client applications.
![demo](https://drop.davy.page/hRMXlkDN/Mar-01-2023%2013-44-04.gif)

[**Terminal Chat client on crates.io**](https://crates.io/crates/tchat)

## How to run the terminal chat server?
The easiest way to do it is to build a docker image and then run it. There's already a [ready-to-use Dockerfile](https://github.com/IDSaves/terminal-chat/blob/master/server/Dockerfile) so you just go with a `docker build -t <imagename> .` inside a server's directory. After you built a docker image just type in `docker run <imagename> -p <your port>:8080`.

If you don't wanna use docker you can install the server's package directly on your computer by typing `cargo install`. Of course you will need to install Rust before you do it :).
## How to use the terminal chat client?
0. Install the app via cargo package manager
```cargo install tchat```
1. Learn options 
```tchat -h```
2. Connect to a server 
```tchat -a <address>``` 
Example server: ```tchat -a 31.172.76.176:9005```

You can use main terminal chat server just to test how it works :). It's address is **31.172.76.176:9005**.