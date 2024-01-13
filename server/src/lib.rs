use std::sync::mpsc;
use std::{
    sync::{Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    _workers: Vec<Worker>,
    job_sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let handle = thread::Builder::new().name(format!("Thread_{}", id)).spawn(move || {
            loop {
                let guard= match receiver.lock() {
                    Err(e) => {
                        panic!("Mutex got poisoned: {e}");
                    },
                    Ok(g) => g,
                };

                let result = guard.recv();
                drop(guard);
                match result {
                    Ok(job) => {
                        println!("Thread {id}: Received job");
                        job()
                    },
                    Err(e) => {
                        eprintln!("Thread {id}: Got error {e}, shutting down");
                        return;
                    }
                };
            }
        })
        .unwrap();
        Worker { id, handle: Some(handle) }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        println!("Cleaning up worker {}", self.id);
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
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

        let workers: Vec<_> = (0..num_threads)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        ThreadPool {
            _workers: workers,
            job_sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Err(e) = self.job_sender.as_ref().expect("Sender is valid").send(Box::new(f)) {
            eprintln!("Unable to process request: {e}");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Dropping the thread pool");
        self.job_sender.take();
    }
}
