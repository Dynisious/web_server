//! `threading` is a module responsible for managing the underlying threading of an instance of a Web Server
//! and the passing of work to the Worker threads of the server.
//!
//! #Last Modified
//!
//! Author --- Daniel Bechaz</br>
//! Date --- 06/09/2017
use std::ops::FnOnce;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
pub use std::result::Result;

/// A `WorkerPool` is a group of threads which can be passed function pointers to execute asynchronously.
pub struct WorkerPool {
    workers: Vec<Worker>,
    sender: Sender<Message>
}

/// A `Message` is the range of messages that can be passed to a `WorkerPool`.
enum Message {
    Message(Job),
    Terminate
}

/// A `FnBox` is a trait which is intended to make a call on a boxed instance of iteself.
trait FnBox {
    /// Makes a call on the Boxed instance of itself.
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

/// A `Job` is a Boxed function pointer that can be called from it's boxed instance.
type Job = Box<FnBox + Send + 'static>;

impl WorkerPool {
    /// Returns a new `WorkerPool` ready to receive messages.
    ///
    /// # Params
    ///
    /// size --- A natural number indicating how many threads the WorkerPool should run.
    pub fn new(size: usize) -> WorkerPool {
        assert!(size > 0, "A `WorkerPool` must have at least one Thread.");
        
        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers: Vec<Worker> = Vec::with_capacity(size);
        
        for id in 0..size {
            workers.push(Worker::new(id, receiver.clone()));
        }
        
        WorkerPool { workers, sender }
    }
    /// Returns the `Result` of sending the passed function to the `WorkerPool`.
    ///
    /// # Params
    ///
    /// job --- The function to have performed asynchronously by the `WorkerPool`.
    pub fn send_job<F>(&mut self, job: F) -> Result<(), &'static str>
        where F: FnOnce() + Send + 'static 
    {
        match self.sender.send(Message::Message(Box::new(job))) {
            Ok(_) => Ok(()),
            Err(_) =>  Err("Cannot pass job to `WorkerPool` (no `Receiver` attached).")
        }
    }
    /// Terminates all `Worker` threads in the `WorkerPool`. In the event of an `Err` when
    /// telling a `Worker` to terminate, the `Err` is returned.
    pub fn shutdown(&mut self) -> Result<(), &'static str> {
        for _ in &mut self.workers {
            if let Err(_) = self.sender.send(Message::Terminate) {
                return Err("Error while sending terminate signal to `Worker`. (No `Receiver` attached)");
            }
        }
        Ok(())
    }
}

impl Drop for WorkerPool {
    /// Cleanly terminates all `Worker`s before the `WorkerPool` is cleaned up.
    fn drop(&mut self) {
        if let Ok(_) = self.shutdown() {
            for worker in &mut self.workers {
                if let Some(thread) = worker.thread.take() {
                    thread.join()
                        .expect(format!("`WorkerPool` failed while joining worker{}.", worker.id).as_str());
                }
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    /// Creates a new `Worker` instance with the passed `id` and using the shared `Receiver`.
    /// `Workers` will loop forever until they receive a `Message::Terminate` signal.
    ///
    /// # Params
    ///
    /// id --- The ID number associated with this `Worker`.<br/>
    /// receiver --- The shared `Receiver` used to get jobs to execute.
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Worker {
        let thread = Some(
            thread::spawn(
                move || {
                    loop {
                        let message = receiver.lock()
                            .expect(format!("Worker{} failed while locking the Receiver.", id).as_str())
                            .recv()
                            .expect(format!("Worker{} failed while receiving a message.", id).as_str());
                        
                        match message {
                            Message::Message(job) => job.call_box(),
                            Message::Terminate => break
                        }
                    }
                }
            )
        );
        
        Worker { id, thread }
    }
}
