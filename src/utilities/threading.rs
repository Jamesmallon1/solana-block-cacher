use std::sync::{Arc, mpsc, Mutex};
use std::thread;

/// Represents a job to be executed by the thread pool.
///
/// This type encapsulates a closure that is sent to worker threads for execution.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// Represents a worker in the thread pool.
///
/// Each worker is a thread running in a loop, waiting to receive and execute jobs.
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    /// Creates a new Worker instance with the given id.
    ///
    /// The worker will listen for incoming jobs on the provided receiver channel and execute them.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the worker.
    /// * `receiver` - The shared receiver channel from which the worker will receive jobs.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `Worker`.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a job; executing.", id);
            job();
        });

        Worker { id, thread }
    }
}

/// A thread pool for executing jobs in parallel.
///
/// This struct manages a pool of worker threads and provides a way to execute tasks concurrently.
struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    /// Creates a new ThreadPool with the specified number of threads.
    ///
    /// Initializes a pool of workers and sets up a channel for sending jobs to these workers.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of worker threads in the pool.
    ///
    /// # Panics
    ///
    /// This function will panic if the specified size is zero.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `ThreadPool`.
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// Executes a job in the thread pool.
    ///
    /// This method takes a closure and sends it to an available worker in the pool for execution.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to execute. This closure must be Send and 'static, as it is executed in a different thread.
    ///
    /// # Panics
    ///
    /// This function will panic if the job cannot be sent to the worker threads. This usually happens
    /// if the receiving side of the channel has been closed, which could indicate that the workers have panicked.
    fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}