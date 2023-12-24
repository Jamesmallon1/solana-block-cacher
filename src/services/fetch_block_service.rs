use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use log::info;
use solana_client::rpc_client::RpcClient;
use crate::utilities::rate_limiter::RateLimiter;
use crate::utilities::threading::ThreadPool;

pub struct FetchBlockService {
    rate_limiter: Arc<Mutex<RateLimiter>>,
    thread_pool: ThreadPool
}

impl FetchBlockService {
    pub fn new(rate_limiter: Arc<Mutex<RateLimiter>>, thread_pool: ThreadPool) -> Self {
        FetchBlockService {
            rate_limiter,
            thread_pool
        }
    }

    pub fn fetch_blocks(&self, from_block: u64, to_block: u64, rpc_url: &str) {
        let no_of_threads = self.thread_pool.get_number_of_workers();
        let total_blocks = to_block - from_block + 1;
        let blocks_per_thread = total_blocks / no_of_threads as u64;
        let completed_count = Arc::new(Mutex::new(0));

        for i in 0..no_of_threads {
            // calculate the range of blocks for each thread
            let start_block = from_block + i as u64 * blocks_per_thread;
            let end_block = if i == no_of_threads - 1 {
                to_block
            } else {
                start_block + blocks_per_thread - 1
            };

            // clone necessary variables prior to movement
            let completed_clone = completed_count.clone();
            let rpc_str = rpc_url.to_string();
            let rl_clone = self.rate_limiter.clone();

            // create the closure
            let closure = move || {
                let rpc_client = RpcClient::new(rpc_str);
                for block_number in start_block..=end_block {
                    while rl_clone.lock().unwrap().should_wait() {
                        thread::sleep(Duration::from_millis(50));
                    }
                    let block = rpc_client.get_block(block_number);
                    info!("Fetched block {}", block_number);
                }
                let mut completed = completed_clone.lock().unwrap();
                *completed += 1;
            };

            // execute the closure in the thread pool
            self.thread_pool.execute(closure);
        }

        loop {
            let completed = completed_count.lock().unwrap();
            if *completed == no_of_threads as u32 {
                break;
            }
            drop(completed);
            thread::sleep(Duration::from_millis(100));
        }
    }
}