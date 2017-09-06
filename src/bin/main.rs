#![doc(hidden)]

extern crate web_server;

use web_server::server::*;
use web_server::http::*;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut srv = Server::start("127.0.0.1:8080", 4,
        move |listener, mut workers, receiver, _| {
            listener.set_nonblocking(true)
                .expect("Server cannot be set to nonblocking.");
            
            loop {
                sleep(Duration::new(0, 250));
                if let Ok((stream, _)) = listener.accept() {
                    workers.send_job(
                        || {
                            handle_connection(stream);
                        }
                    ).expect("Failed to send job to WorkerPool.");
                }
                
                if let Ok(Message::Shutdown) = receiver.try_recv() {
                    if let Err(e) = workers.shutdown() {
                        panic!(e);
                    }
                    break;
                }
            }
        },
    ());
    
    loop {
        let mut command = String::new();
        io::stdin().read_line(&mut command)
            .expect("Failed to read user input.");
        
        let command = command.trim().to_lowercase();
        if command.as_str() == "shutdown" {
            while !srv.shutdown() {}
            break;
        } else {
            print!("Did not recognise command '");
            io::stdout().write(command.as_bytes()).expect("Error writing to standard output.");
            print!("'");
        }
    }
    
    srv.join()
        .expect("Failed to join on the Server.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    if let Ok(_) = stream.read(&mut buffer) {
        let message = MessageHTTP::from_utf8(buffer.to_vec()).unwrap();
        
        let (status_line, filename) = if let ("GET", target, _) = message.start_line.request() {
            if target == "/" {
                ("HTTP/1.1 200 OK\r\n\r\n", String::from("html/index.html"))
            } else {
                ("HTTP/1.1 200 OK\r\n\r\n", format!("html{}.html", target))
            }
        } else {
            ("HTTP/1.1 404 NOT FOUND\r\n\r\n", String::from("html/404.html"))
        };

        if let Ok(mut file) = File::open(filename) {
            let mut contents = String::new();

            if let Ok(_) = file.read_to_string(&mut contents) {
                let response = format!("{}{}", status_line, contents);

                if let Ok(_) = stream.write(response.as_bytes()) {
                    stream.flush().expect("Error sending response to client.");
                }
            }
        } else if let Ok(mut file) = File::open("html/404.html") {
            let status_line = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
            let mut contents = String::new();

            if let Ok(_) = file.read_to_string(&mut contents) {
                let response = format!("{}{}", status_line, contents);

                if let Ok(_) = stream.write(response.as_bytes()) {
                    stream.flush().expect("Error sending response to client.");
                }
            }
        }
    }
}
