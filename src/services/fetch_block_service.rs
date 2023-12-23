use std::sync::{Arc, Mutex};
use log::info;
use solana_client::rpc_client::RpcClient;
use crate::utilities::rate_limiter::RateLimiter;
use crate::utilities::threading::ThreadPool;

pub struct FetchBlockService {
    rpc_client: Arc<RpcClient>,
    rate_limiter: Mutex<RateLimiter>,
    thread_pool: ThreadPool
}

impl FetchBlockService {
    pub fn new(rpc_url: &str, rate_limiter: Mutex<RateLimiter>, thread_pool: ThreadPool) -> Self {
        FetchBlockService {
            rpc_client: Arc::new(RpcClient::new(rpc_url.to_string())),
            rate_limiter,
            thread_pool
        }
    }

    pub fn fetch_blocks(&self, from_block: u64, to_block: u64) {
        let no_of_threads = self.thread_pool.get_number_of_workers();
        let total_blocks = to_block - from_block + 1;
        let blocks_per_thread = total_blocks / no_of_threads as u64;

        for i in 0..no_of_threads {
            // calculate the range of blocks for each thread
            let start_block = from_block + i as u64 * blocks_per_thread;
            let end_block = if i == no_of_threads - 1 {
                to_block
            } else {
                start_block + blocks_per_thread - 1
            };

            // clone the RPC client for each thread
            let rpc_client_clone = self.rpc_client.clone();

            // create the closure
            let closure = move || {
                for block_number in start_block..=end_block {
                    let block = rpc_client_clone.get_block(block_number as u64);
                    info!("Fetched block {}", block_number);
                }
            };

            // execute the closure in the thread pool
            self.thread_pool.execute(closure);
        }
    }
}