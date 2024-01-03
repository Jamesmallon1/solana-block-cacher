use crate::networking::BlockFetcher;
use log::{debug, info};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;

/// Represents a job to be executed by the thread pool.
///
/// This type encapsulates a closure that is sent to worker threads for execution.
type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

/// Represents a worker in the thread pool.
///
/// Each worker is a thread running in a loop, waiting to receive and execute jobs.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
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
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            if let Message::NewJob(job) = message {
                debug!("Worker {} got a job; executing.", id);
                job();
            } else {
                debug!("Terminating worker {}", id);
                break;
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }

    /// Joins the worker's thread.
    ///
    /// This method takes the thread out of the `Worker` and joins it, ensuring that it completes its execution.
    /// It's used to gracefully shut down the worker thread, making sure that all the tasks assigned to this
    /// worker are finished before the thread is terminated. If the worker's thread is already joined or never
    /// started, this method does nothing.
    ///
    /// # Panics
    ///
    /// This method will panic if the thread's `join` call panics, which might occur if the thread has already
    /// been joined elsewhere or if the thread panics while trying to join.
    fn join(&mut self) {
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

/// A thread pool for executing jobs in parallel.
///
/// This struct manages a pool of worker threads and provides a way to execute tasks concurrently.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

pub trait JobDispatcher {
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
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;

    /// Gracefully shuts down the thread pool.
    ///
    /// This method sends a termination message to each worker in the pool, instructing them to stop processing
    /// further jobs. It then proceeds to join each worker's thread, ensuring they complete their current task
    /// and terminate gracefully. This method is essential for cleanly shutting down the thread pool and
    /// preventing any resource leaks or unfinished jobs.
    ///
    /// The method iterates through all the workers, sending a `Terminate` message to each, and then joins their
    /// threads. This two-step approach ensures that all workers receive the termination message before the
    /// thread pool starts joining their threads.
    ///
    /// # Panics
    ///
    /// This method will panic if it fails to send the termination message to any of the workers or if joining
    /// any of the worker threads results in a panic. The former might occur if the receiving end of the channel
    /// is disconnected (e.g., if a worker thread panics and exits prematurely), and the latter might occur if
    /// a thread panics during its execution or has already been joined.
    fn destroy(&mut self);
}

pub trait WorkerCounter {
    /// Gets the number of worker threads within the thread pool
    fn get_number_of_workers(&self) -> usize;
}

impl JobDispatcher for ThreadPool {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }

    fn destroy(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            debug!("Shutting down worker {}", worker.id);
            worker.join();
        }
    }
}

impl WorkerCounter for ThreadPool {
    fn get_number_of_workers(&self) -> usize {
        self.workers.len()
    }
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
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel::<Message>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }
}

/// Calculates the optimum number of worker threads for fetching blockchain blocks.
///
/// This function estimates the ideal number of threads to use for fetching blocks
/// based on a given rate limit and time window. It uses a `BlockFetcher` client
/// to sample block retrieval times and calculates the average time per request.
/// The function then determines the maximum number of effective requests per thread
/// and calculates the optimum number of threads based on the rate limit and
/// the time window.
///
/// # Parameters
/// - `client`: A boxed trait object implementing `BlockFetcher` and `Send`. This
///   client is used to fetch blockchain blocks and measure request times.
/// - `rate_limit`: The maximum number of requests allowed per unit of time (u32).
/// - `window`: The time window in seconds (u32) for the rate limit.
///
/// # Returns
/// Returns the optimum number of threads (`usize`) calculated based on the rate
/// limit and average time to retrieve a block.
///
/// # Examples
/// ```
/// let client = Box::new(MyBlockFetcher::new());
/// let rate_limit = 100;
/// let window = 60;
/// let optimum_threads = get_optimum_number_of_threads(client, rate_limit, window);
/// println!("Optimum number of threads: {}", optimum_threads);
/// ```
///
/// # Notes
/// - The function currently uses a fixed sample size of 3 for estimating the
///   average block retrieval time. This might be altered in the future to allow
///   for a range of slots sampled from the user's range.
/// - The function logs information about the calculation process and the
///   determined optimum number of threads.
pub fn get_optimum_number_of_threads(client: Box<dyn BlockFetcher + Send>, rate_limit: u32, window: u32) -> usize {
    info!("Calculating the optimum number of worker threads to use");
    let sample_size = 3;

    // calculate the average time to retrieve a block
    // todo: alter to allow for a range of slots sampled form the user's range
    let avg_time_per_request_ms = (0..sample_size)
        .map(|_| {
            let start = Instant::now();
            client.get_block(5003).unwrap();
            start.elapsed().as_millis() as u32
        })
        .sum::<u32>()
        / sample_size;

    // calculate the optimum number of threads
    let window_ms = window * 1000;
    let effective_requests_per_thread = (window_ms as f64 / avg_time_per_request_ms as f64).min(rate_limit as f64);
    let optimum_threads = (rate_limit as f64 / effective_requests_per_thread).ceil() as usize;
    info!("Utilising {} threads to pull blocks", optimum_threads);

    optimum_threads
}

// mocking for unit tests
pub struct MockThreadPool {
    number_of_workers: usize,
}
impl JobDispatcher for MockThreadPool {
    fn execute<F>(&self, _: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }

    fn destroy(&mut self) {}
}

impl WorkerCounter for MockThreadPool {
    fn get_number_of_workers(&self) -> usize {
        self.number_of_workers
    }
}

impl MockThreadPool {
    pub fn new(number_of_workers: usize) -> Self {
        MockThreadPool { number_of_workers }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::networking::BlockFetcherFactory;

    #[test]
    fn worker_new_test() {
        let (_, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let worker = Worker::new(1, receiver);

        assert_eq!(worker.id, 1);
    }

    #[test]
    fn test_join_normal_termination() {
        let (tx, rx) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(rx));
        let mut worker = Worker::new(1, receiver.clone());

        tx.send(Message::NewJob(Box::new(|| {}))).unwrap();
        tx.send(Message::Terminate).unwrap();

        worker.join();
    }

    #[test]
    fn test_join_repeated_calls() {
        let (tx, rx) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(rx));
        let mut worker = Worker::new(2, receiver.clone());

        tx.send(Message::NewJob(Box::new(|| {}))).unwrap();
        tx.send(Message::Terminate).unwrap();

        // first join call
        worker.join();

        // second join call should not panic
        worker.join();
    }

    #[test]
    fn test_join_with_no_active_thread() {
        let mut worker = Worker { id: 3, thread: None };

        // join should complete without panic
        worker.join();
    }

    #[test]
    fn threadpool_new_test() {
        let pool = ThreadPool::new(3);
        assert_eq!(pool.workers.len(), 3);
    }

    #[test]
    fn threadpool_execute_test() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let mut pool = ThreadPool::new(3);
        let job_count = Arc::new(AtomicUsize::new(0));

        for _ in 0..10 {
            let job_count = Arc::clone(&job_count);
            pool.execute(move || {
                job_count.fetch_add(1, Ordering::SeqCst);
            });
        }

        pool.destroy();

        assert_eq!(job_count.load(Ordering::SeqCst), 10);
    }

    fn run_dummy_jobs(pool: &ThreadPool, n: usize) {
        for _ in 0..n {
            pool.execute(|| {
                // dummy job, does nothing but ensures workers are busy
            });
        }
    }

    #[test]
    fn test_destroy_thread_pool() {
        let mut pool = ThreadPool::new(4);
        run_dummy_jobs(&pool, 8);

        // no panic expected
        pool.destroy();
    }

    #[test]
    fn test_get_number_of_workers() {
        let mut pool = ThreadPool::new(3);
        assert_eq!(pool.get_number_of_workers(), 3);

        // destroy the pool to avoid lingering threads
        pool.destroy();
    }

    #[test]
    fn test_get_optimum_number_of_threads() {
        let block_fetcher = BlockFetcherFactory::new(true, "").create_block_fetcher();
        let result = get_optimum_number_of_threads(block_fetcher, 50, 1);

        // this is possible due to constant time of 250ms latency in the mocked block fetcher
        assert_eq!(result, 13);
    }
}
