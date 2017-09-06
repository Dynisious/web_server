//! `server` is a module responsible for the construction, destruction and interfacing with an active instance of a Web Server.
//!
//! #Last Modified
//!
//! Author --- Daniel Bechaz</br>
//! Date --- 06/09/2017

pub use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender, Receiver};
pub use std::sync::mpsc::SendError;
use super::threading::*;
use std::thread;
use std::any::Any;

/// A `Server` is an independant thread which handles concurrent connections using multiple `Worker` threads.
pub struct Server {
    /// A handler to the `Server`s own thread.
    server: Option<thread::JoinHandle<()>>,
    /// A `Sender` to signal the `Server` thread.
    pub sender: Sender<Message>
}

/// `Message`s to send to the `Server` thread.
pub enum Message {
    /// A generic code message to allow customisation by the user.
    Code(u32),
    /// A Shutdown Message to signal the `Server` to shutdown.
    Shutdown
}

impl Server {
    /// Returns a new `Server` with a listener bound the passed address and running the passed main function on `Server`.
    ///
    /// # Params
    ///
    /// addr --- The address to bind the `TcpListener` too.</br>
    /// workers --- The number of `Worker` threads to spawn.</br>
    /// server --- The main loop for the `Server`.</br>
    /// args --- The arguments to pass to the servers main function.
    pub fn start<A: Send + 'static, F>(addr: &str, workers: usize, server: F, args: A) -> Server
        where F: FnOnce(TcpListener, WorkerPool, Receiver<Message>, A) + Send + 'static
    {
        let listener = TcpListener::bind(addr)
            .expect("Failed to bind to `addr`.");
        let workers = WorkerPool::new(workers);
        let (sender, receiver) = channel();
        let server = Some(
            thread::spawn(
                move || {
                    server(listener, workers, receiver, args)
                }
            )
        );
        
        Server { server, sender }
    }
    /// Blocks the calling thread until the `Server`s main thread terminates.
    pub fn join(&mut self) -> Result<(), Box<Any + Send + 'static>> {
        self.server.take().unwrap().join()
    }
    /// Sends the `Shutdown` `Message` to the `Server` thread.</br>
    /// Returns `true` if the send succeeded and the `Server` thread joined, else it returns `false`.
    pub fn shutdown(&mut self) -> bool {
        match self.sender.send(Message::Shutdown) {
            Ok(_) => true,
            Err(_) => false
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.shutdown();
    }
}
