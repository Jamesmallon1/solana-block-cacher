use crate::model::solana_block;
use crate::model::solana_block::{BlockBatch, Reverse, SerializedSolanaBlock};
use crate::utilities::priority_queue::PriorityQueue;
use crate::utilities::rate_limiter::RateLimiter;
use crate::utilities::threading::ThreadPool;
use log::{debug, error, info};
use solana_client::rpc_client::RpcClient;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

/// `FetchBlockService` is responsible for fetching blocks from a Solana blockchain.
/// It utilizes a thread pool to parallelize the fetching process, a rate limiter
/// to control the request rate, and a priority queue for organizing the fetched blocks.
///
/// # Fields
/// - `write_queue`: A thread-safe priority queue to store the fetched blocks.
/// - `rate_limiter`: A rate limiter to control the frequency of fetch requests.
/// - `thread_pool`: A thread pool for concurrent block fetching.
/// - `condvar`: A condition variable used for thread synchronization.
pub struct FetchBlockService {
    write_queue: Arc<Mutex<PriorityQueue<Reverse<BlockBatch>>>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    thread_pool: ThreadPool,
    condvar: Arc<Condvar>,
}

impl FetchBlockService {
    /// Creates a new instance of `FetchBlockService`.
    ///
    /// # Parameters
    /// - `write_queue`: Thread-safe priority queue for storing fetched blocks.
    /// - `rate_limiter`: Rate limiter for controlling request frequency.
    /// - `thread_pool`: Thread pool for concurrent fetching.
    /// - `condvar`: Condition variable for thread synchronization.
    ///
    /// # Returns
    /// Returns a new `FetchBlockService` instance.
    pub fn new(
        write_queue: Arc<Mutex<PriorityQueue<Reverse<BlockBatch>>>>,
        rate_limiter: Arc<Mutex<RateLimiter>>,
        thread_pool: ThreadPool,
        condvar: Arc<Condvar>,
    ) -> Self {
        FetchBlockService {
            write_queue,
            rate_limiter,
            thread_pool,
            condvar,
        }
    }

    /// Fetches blocks from the Solana blockchain in a range from `from_slot` to `to_slot`
    /// and processes them in parallel using the thread pool. The fetched blocks are stored
    /// in the priority queue.
    ///
    /// # Parameters
    /// - `from_slot`: The starting slot number for fetching blocks.
    /// - `to_slot`: The ending slot number for fetching blocks.
    /// - `rpc_url`: The URL of the Solana RPC client.
    ///
    /// # Remarks
    /// This method calculates the number of slots to be processed per thread and then
    /// dispatches multiple threads to fetch and process the blocks concurrently.
    pub fn fetch_blocks(&mut self, from_slot: u64, to_slot: u64, rpc_url: &str) {
        // calculate slots per thread and initialize thread completion counter
        let no_of_threads = self.thread_pool.get_number_of_workers();
        let total_slots = to_slot - from_slot + 1;
        let slots_per_thread = total_slots / no_of_threads as u64;
        let completed_count = Arc::new(Mutex::new(0));
        let number_of_block_batches =
            ((slots_per_thread as f64 / solana_block::BATCH_SIZE as f64).ceil() as u64).max(1);

        for i in 0..no_of_threads {
            // clone necessary variables prior to movement
            let completed_clone = Arc::clone(&completed_count);
            let rpc_str = rpc_url.to_string();
            let rl_clone = Arc::clone(&self.rate_limiter);
            let queue_clone = Arc::clone(&self.write_queue);
            let condvar_clone = Arc::clone(&self.condvar);

            let closure = move || {
                for batch_number in 1..=number_of_block_batches {
                    let (mut current_slot, mut end_slot) = FetchBlockService::calculate_batch_start_and_end_slots(
                        from_slot,
                        to_slot,
                        batch_number,
                        i as u64,
                        no_of_threads as u64,
                    );
                    let mut current_batch = BlockBatch::new(from_slot as f64, current_slot as f64);
                    FetchBlockService::populate_batch(
                        &mut current_batch,
                        &mut current_slot,
                        &mut end_slot,
                        rl_clone.clone(),
                        &rpc_str,
                    );
                    debug!(
                        "Dispatching block batch {}-{} to be written to file.",
                        current_slot - solana_block::BATCH_SIZE + 1,
                        current_slot
                    );
                    queue_clone.lock().unwrap().push(Reverse(current_batch));
                    condvar_clone.notify_one();
                }

                *completed_clone.lock().unwrap() += 1;
            };

            self.thread_pool.execute(closure);
        }

        info!("Block caching is starting, please wait..");
        self.wait_for_thread_pool_completion(completed_count, no_of_threads);

        // cleanup
        self.thread_pool.destroy();
    }

    fn populate_batch(
        batch: &mut BlockBatch,
        current_slot: &mut u64,
        end_slot: &mut u64,
        rate_limiter: Arc<Mutex<RateLimiter>>,
        rpc_str: &str,
    ) {
        let rpc_client = RpcClient::new(rpc_str);
        while current_slot <= end_slot {
            while rate_limiter.lock().unwrap().should_wait() {
                thread::sleep(Duration::from_millis(10));
            }
            let block_result = rpc_client.get_block(*current_slot + 1_u64);
            match block_result {
                Ok(block) => {
                    let serialized_data_result = serde_json::to_string(&block);
                    match serialized_data_result {
                        Ok(data) => {
                            let solana_block = SerializedSolanaBlock {
                                slot_number: block.parent_slot as u64,
                                data,
                            };
                            batch.push(solana_block);
                        }
                        Err(err) => {
                            error!(
                                "An error occurred serializing a Solana block: {}, Error: {}",
                                current_slot, err
                            );
                        }
                    }
                }
                Err(err) => {
                    error!("Could not retrieve block {} due to error: {}", current_slot, err);
                }
            }
            *current_slot += 1;
        }
    }

    fn calculate_batch_start_and_end_slots(
        from_slot: u64,
        to_slot: u64,
        batch_number: u64,
        thread_number: u64,
        total_threads: u64,
    ) -> (u64, u64) {
        let start_slot = from_slot
            + (thread_number * solana_block::BATCH_SIZE)
            + ((batch_number - 1) * solana_block::BATCH_SIZE * total_threads);
        let end_slot = (start_slot + solana_block::BATCH_SIZE - 1).min(to_slot);

        (start_slot, end_slot)
    }

    fn wait_for_thread_pool_completion(&self, completed_count: Arc<Mutex<i32>>, no_of_threads: usize) {
        loop {
            let completed = completed_count.lock().unwrap();
            if *completed == no_of_threads as i32 {
                break;
            }
            drop(completed);
            thread::sleep(Duration::from_millis(500));
        }
    }
}
