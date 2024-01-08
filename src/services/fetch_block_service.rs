use crate::model::solana_block;
use crate::model::solana_block::{BlockBatch, Reverse};
use crate::networking::{BlockFetcher, BlockFetcherFactory};
use crate::utilities::priority_queue::Queue;
use crate::utilities::rate_limiter::RateLimiting;
use crate::utilities::threading::{JobDispatcher, WorkerCounter};
use log::{debug, info};
use std::error::Error;
use std::sync::atomic::{AtomicI32, Ordering};
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
pub struct FetchBlockService<
    R: RateLimiting,
    P: for<'a> Queue<'a, Reverse<BlockBatch>>,
    T: JobDispatcher + WorkerCounter,
> {
    write_queue: Arc<Mutex<P>>,
    rate_limiter: Arc<Mutex<R>>,
    thread_pool: T,
    client_factory: BlockFetcherFactory,
    condvar: Arc<Condvar>,
}

impl<
        R: RateLimiting + Send + 'static,
        P: for<'a> Queue<'a, Reverse<BlockBatch>> + Send + 'static,
        T: JobDispatcher + WorkerCounter,
    > FetchBlockService<R, P, T>
{
    /// Creates a new instance of `FetchBlockService`.
    ///
    /// # Parameters
    /// - `write_queue`: Thread-safe priority queue for storing fetched blocks.
    /// - `rate_limiter`: Rate limiter for controlling request frequency.
    /// - `thread_pool`: Thread pool for concurrent fetching.
    /// - `rpc_client`: A solana RPC client to fetch blocks.
    /// - `condvar`: Condition variable for thread synchronization.
    ///
    /// # Returns
    /// Returns a new `FetchBlockService` instance.
    pub fn new(
        write_queue: Arc<Mutex<P>>,
        rate_limiter: Arc<Mutex<R>>,
        thread_pool: T,
        condvar: Arc<Condvar>,
        client_factory: BlockFetcherFactory,
    ) -> Self {
        FetchBlockService {
            write_queue,
            rate_limiter,
            thread_pool,
            condvar,
            client_factory,
        }
    }

    /// Fetches blocks from the Solana blockchain in a range from `from_slot` to `to_slot`
    /// and processes them in parallel using the thread pool. The fetched blocks are stored
    /// in the priority queue.
    ///
    /// # Parameters
    /// - `from_slot`: The starting slot number for fetching blocks.
    /// - `to_slot`: The ending slot number for fetching blocks.
    ///
    /// # Remarks
    /// This method calculates the number of slots to be processed per thread and then
    /// dispatches multiple threads to fetch and process the blocks concurrently.
    pub fn fetch_blocks(&mut self, from_slot: u64, to_slot: u64) {
        // calculate slots per thread and initialize thread completion counter
        let no_of_threads = self.thread_pool.get_number_of_workers();
        let total_slots = to_slot - from_slot + 1;
        let slots_per_thread = total_slots / no_of_threads as u64;
        let completed_count = Arc::new(AtomicI32::new(0));
        let number_of_block_batches =
            ((slots_per_thread as f64 / solana_block::BATCH_SIZE as f64).ceil() as u64).max(1);

        for i in 0..no_of_threads {
            // clone necessary variables prior to movement
            let completed_clone = Arc::clone(&completed_count);
            let rl_clone = Arc::clone(&self.rate_limiter);
            let queue_clone = Arc::clone(&self.write_queue);
            let condvar_clone = Arc::clone(&self.condvar);
            let rpc_client = self.client_factory.create_block_fetcher();

            let closure = move || {
                for batch_number in 1..=number_of_block_batches {
                    let (mut current_slot, mut end_slot) =
                        match FetchBlockService::<R, P, T>::calculate_batch_start_and_end_slots(
                            from_slot,
                            to_slot,
                            batch_number,
                            i as u64,
                            no_of_threads as u64,
                        ) {
                            Ok(slots) => slots,
                            Err(_) => break,
                        };

                    let mut current_batch = BlockBatch::new(from_slot as f64, current_slot as f64);
                    FetchBlockService::<R, P, T>::populate_batch(
                        &mut current_batch,
                        &mut current_slot,
                        &mut end_slot,
                        rl_clone.clone(),
                        &rpc_client,
                    );
                    debug!(
                        "Dispatching block batch {}-{} to be written to file.",
                        current_slot - solana_block::BATCH_SIZE + 1,
                        current_slot
                    );
                    queue_clone.lock().unwrap().push(Reverse(current_batch));
                    condvar_clone.notify_one();
                }

                completed_clone.fetch_add(1, Ordering::Relaxed);
            };

            self.thread_pool.execute(closure);
        }

        info!("Block caching is starting, please wait..");
        self.wait_for_thread_pool_completion(completed_count, no_of_threads);

        // cleanup
        self.thread_pool.destroy();
    }

    fn populate_batch<RL: RateLimiting>(
        batch: &mut BlockBatch,
        current_slot: &mut u64,
        end_slot: &mut u64,
        rate_limiter: Arc<Mutex<RL>>,
        rpc_client: &Box<dyn BlockFetcher + Send>,
    ) {
        while current_slot <= end_slot {
            while rate_limiter.lock().unwrap().should_wait() {
                // todo: is there a better way to do this synchronously?
                thread::sleep(Duration::from_millis(10));
            }
            let block_result = rpc_client.get_block(*current_slot + 1_u64);
            match block_result {
                Ok(solana_block) => batch.push(solana_block),
                Err(_) => {
                    // todo: we can handle the error here so that in the future we can go back and
                    // todo: insert missed blocks into the json, we can prevent most of these scenarios by introducing
                    // todo: a retry mechanism
                    // ideas: post-processing insertion and sorting, add more
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
    ) -> Result<(u64, u64), StartSlotExceedsEndSlotError> {
        let start_slot = from_slot
            + (thread_number * solana_block::BATCH_SIZE)
            + ((batch_number - 1) * solana_block::BATCH_SIZE * total_threads);
        let end_slot = (start_slot + solana_block::BATCH_SIZE - 1).min(to_slot);

        if start_slot > to_slot {
            return Err(StartSlotExceedsEndSlotError);
        }

        Ok((start_slot, end_slot))
    }

    fn wait_for_thread_pool_completion(&self, completed_count: Arc<AtomicI32>, no_of_threads: usize) {
        loop {
            if completed_count.load(Ordering::SeqCst) == no_of_threads as i32 {
                break;
            }
            thread::sleep(Duration::from_millis(500));
        }
    }
}

#[derive(Debug)]
struct StartSlotExceedsEndSlotError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::fetch_block_service::FetchBlockService;
    use crate::utilities::priority_queue::MockQueue;
    use crate::utilities::rate_limiter::MockRateLimiting;
    use crate::utilities::threading::MockThreadPool;
    use std::sync::{Arc, Condvar, Mutex};
    use std::time::Instant;

    #[test]
    fn test_new_fetch_block_service() {
        // mock
        let mock_write_queue = Arc::new(Mutex::new(MockQueue::<Reverse<BlockBatch>>::new()));
        let mock_rate_limiter = Arc::new(Mutex::new(MockRateLimiting::default()));
        let mock_thread_pool = MockThreadPool::new(2);
        let mock_condvar = Arc::new(Condvar::new());
        let block_fetcher = BlockFetcherFactory::new(true, "");

        let service = FetchBlockService::new(
            mock_write_queue.clone(),
            mock_rate_limiter.clone(),
            mock_thread_pool,
            mock_condvar.clone(),
            block_fetcher,
        );

        // assert service is initialized with same dependencies
        assert!(Arc::ptr_eq(&service.write_queue, &mock_write_queue));
        assert!(Arc::ptr_eq(&service.rate_limiter, &mock_rate_limiter));
    }

    #[test]
    fn test_calculate_batch_start_and_end_slots_even_distribution() {
        let from_slot = 0;
        let to_slot = 99;
        let batch_number = 1;
        let thread_number = 0;
        let total_threads = 1;

        let (start_slot, end_slot) = FetchBlockService::<
            MockRateLimiting,
            MockQueue<Reverse<BlockBatch>>,
            MockThreadPool,
        >::calculate_batch_start_and_end_slots(
            from_slot, to_slot, batch_number, thread_number, total_threads
        )
        .unwrap();

        assert_eq!(start_slot, 0);
        assert_eq!(end_slot, 49);
    }

    #[test]
    fn test_calculate_batch_start_and_end_slots_even_distribution_should_error() {
        let from_slot = 0;
        let to_slot = 99;
        let batch_number = 3;
        let thread_number = 0;
        let total_threads = 1;

        let result = FetchBlockService::<
            MockRateLimiting,
            MockQueue<Reverse<BlockBatch>>,
            MockThreadPool,
        >::calculate_batch_start_and_end_slots(
            from_slot, to_slot, batch_number, thread_number, total_threads
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_batch_start_and_end_slots_uneven_distribution_one_final_slot() {
        let from_slot = 0;
        let to_slot = 100;
        let batch_number = 3;
        let thread_number = 0;
        let total_threads = 1;

        let (start_slot, end_slot) = FetchBlockService::<
            MockRateLimiting,
            MockQueue<Reverse<BlockBatch>>,
            MockThreadPool,
        >::calculate_batch_start_and_end_slots(
            from_slot, to_slot, batch_number, thread_number, total_threads
        )
        .unwrap();

        assert_eq!(start_slot, 100);
        assert_eq!(end_slot, 100);
    }

    #[test]
    fn test_wait_for_thread_pool_completion_all_complete() {
        let mut completed_count = Arc::new(AtomicI32::new(0));
        let no_of_threads = 5;

        // simulate all threads completion
        completed_count.fetch_add(no_of_threads as i32, Ordering::Relaxed);

        let service = FetchBlockService::<MockRateLimiting, MockQueue<Reverse<BlockBatch>>, MockThreadPool>::new(
            Arc::new(Mutex::new(MockQueue::new())),
            Arc::new(Mutex::new(MockRateLimiting::default())),
            MockThreadPool::new(no_of_threads),
            Arc::new(Condvar::new()),
            BlockFetcherFactory::new(true, ""),
        );

        let start = Instant::now();
        service.wait_for_thread_pool_completion(completed_count.clone(), no_of_threads);
        let duration = start.elapsed();

        // assert that there is no waiting
        // on slower computers you may need to adjust duration of assert, choosing 50 as a safe value for now
        assert!(duration < Duration::from_millis(50));
    }

    #[test]
    fn test_wait_for_thread_pool_completion_partial_complete() {
        let mut completed_count = Arc::new(AtomicI32::new(0));
        let no_of_threads = 5usize;

        // simulate one thread has not completed
        completed_count.fetch_add(no_of_threads as i32 - 1, Ordering::Relaxed);

        // spawn a thread to simulate a completion event after 200ms (< 500ms wait time)
        let completed_clone = completed_count.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            completed_clone.fetch_add(1, Ordering::Relaxed);
        });

        let service = FetchBlockService::<MockRateLimiting, MockQueue<Reverse<BlockBatch>>, MockThreadPool>::new(
            Arc::new(Mutex::new(MockQueue::new())),
            Arc::new(Mutex::new(MockRateLimiting::default())),
            MockThreadPool::new(no_of_threads),
            Arc::new(Condvar::new()),
            BlockFetcherFactory::new(true, ""),
        );

        let start = Instant::now();
        service.wait_for_thread_pool_completion(completed_count.clone(), no_of_threads);
        let duration = start.elapsed();

        // assert that the function waited longer than 500ms
        assert!(duration >= Duration::from_millis(500));
    }

    #[test]
    fn test_populate_batch() {
        let mut batch = BlockBatch {
            sequence_number: 1,
            batch: vec![],
        };

        let mut mock_rate_limiter = MockRateLimiting::default();
        mock_rate_limiter.expect_should_wait().return_const(false);
        let fetcher_factory = BlockFetcherFactory::new(true, "");

        FetchBlockService::<MockRateLimiting, MockQueue<Reverse<BlockBatch>>, MockThreadPool>::populate_batch(
            &mut batch,
            &mut 1,
            &mut 99,
            Arc::new(Mutex::new(mock_rate_limiter)),
            &fetcher_factory.create_block_fetcher(),
        );

        // check last batch number contains a 123 default number
        assert_eq!(batch.batch.get(98).unwrap().slot_number, 123);
    }
}
