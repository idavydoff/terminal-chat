# Terminal Chat server

## How to run the terminal chat server?
The easiest way to do it is to build a docker image and then it. There's already a [ready-to-use Dockerfile](https://github.com/IDSaves/terminal-chat/blob/master/server/Dockerfile) so you just go with a `docker build -t <imagegame> .` inside a server's directory. After you built a docker image just type in docker run <imagename> -p <your port>:8080.

If you don't wanna use docker you can install the server's package directly on your computer by typing `cargo install`. Of course you will need to install Rust before you do it :).
