use std::fmt::format;
use std::sync::mpsc;
use std::{
    sync::{Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    job_sender: Box<mpsc::Sender<Job>>,
}

struct Worker {
    handle: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let handle = thread::Builder::new().name(format!("Thread_{}", id)).spawn(move || {
            loop {
                let guard;
                match receiver.lock() {
                    Err(e) => {
                        eprintln!("Mutex got poisoned: {e}");
                        continue;
                    },
                    Ok(g) => guard = g,
                };

                let result = guard.recv();
                drop(guard);
                match result {
                    Ok(job) => {
                        println!("Thread {id}: Received job");
                        job()
                    },
                    Err(e) => eprintln!("Thread {id}: Got error {e}"),
                };
            }
        })
        .unwrap();
        Worker { handle }
    }
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(num_threads: usize) -> ThreadPool {
        assert!(num_threads > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let sender = Box::new(sender);

        let workers: Vec<_> = (0..num_threads)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        ThreadPool {
            workers,
            job_sender: sender,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Err(e) = self.job_sender.send(Box::new(f)) {
            eprintln!("Unable to process request: {e}");
        }
    }
}
