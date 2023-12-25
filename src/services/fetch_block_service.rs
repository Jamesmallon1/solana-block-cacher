use crate::utilities::rate_limiter::RateLimiter;
use crate::utilities::threading::ThreadPool;
use log::{error, info};
use solana_client::rpc_client::RpcClient;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::System;

pub struct FetchBlockService {
    rate_limiter: Arc<Mutex<RateLimiter>>,
    thread_pool: ThreadPool,
}

impl FetchBlockService {
    pub fn new(rate_limiter: Arc<Mutex<RateLimiter>>, thread_pool: ThreadPool) -> Self {
        FetchBlockService {
            rate_limiter,
            thread_pool,
        }
    }

    pub fn fetch_blocks(&self, from_block: u64, to_block: u64, rpc_url: &str) {
        // calculate blocks per thread and initialize thread completion counter
        let no_of_threads = self.thread_pool.get_number_of_workers();
        let total_blocks = to_block - from_block + 1;
        let blocks_per_thread = total_blocks / no_of_threads as u64;
        let completed_count = Arc::new(Mutex::new(0));
        let number_of_blocks_to_pull_per_call = self.calculate_blocks_to_pull_per_call_per_thread(no_of_threads);

        for i in 0..no_of_threads {
            // calculate the range of blocks for each call on each thread
            let start_block = from_block + i as u64 * blocks_per_thread;
            let end_block = if i == no_of_threads - 1_usize {
                to_block
            } else {
                start_block + blocks_per_thread - 1
            };

            let block_number_ranges = self.get_block_range_per_call_per_thread(
                blocks_per_thread,
                number_of_blocks_to_pull_per_call as u64,
                start_block,
                end_block,
                no_of_threads as u64,
                i as u64,
            );

            // clone necessary variables prior to movement
            let completed_clone = completed_count.clone();
            let rpc_str = rpc_url.to_string();
            let rl_clone = self.rate_limiter.clone();

            // create the closure
            let closure = move || {
                let rpc_client = RpcClient::new(rpc_str);
                for block_numbers in block_number_ranges {
                    while rl_clone.lock().unwrap().should_wait() {
                        thread::sleep(Duration::from_millis(50));
                    }

                    let block_result = rpc_client.get_blocks(block_numbers.0, Some(block_numbers.1));
                    match block_result {
                        Ok(block) => {}
                        Err(err) => {
                            error!("Could not retrieve block range o {} due to error: {}", 1, err);
                        }
                    }
                    //info!("Fetched block {}", block_number);
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

    fn get_block_range_per_call_per_thread(
        &self,
        blocks_per_thread: u64,
        number_of_blocks_to_pull_per_call: u64,
        from_block: u64,
        to_block: u64,
        no_of_threads: u64,
        index: u64,
    ) -> Vec<(u64, u64)> {
        return if blocks_per_thread <= number_of_blocks_to_pull_per_call {
            let start_block = from_block + index * blocks_per_thread;
            let end_block = if index == no_of_threads - 1 {
                to_block
            } else {
                start_block + blocks_per_thread - 1
            };
            vec![(start_block, end_block)]
        } else {
            let mut ranges = Vec::new();
            let mut start_block = from_block + index * blocks_per_thread;
            let final_block = if index == no_of_threads - 1 {
                to_block
            } else {
                start_block + blocks_per_thread - 1
            };

            while start_block <= final_block {
                let end_block = std::cmp::min(start_block + number_of_blocks_to_pull_per_call - 1, final_block);
                ranges.push((start_block, end_block));
                start_block = end_block + 1;
            }

            ranges
        };
    }

    fn calculate_blocks_to_pull_per_call_per_thread(&self, no_of_threads: usize) -> usize {
        info!("Calculating maximum number of blocks to pull per call");
        let available_memory_in_kb = self.get_available_memory_in_kb();
        let avg_size_of_block_kb = self.estimate_size_of_block_in_kb();
        // todo: potentially allow this buffer to be configurable by the user for low mem users they may want to
        // todo: increase this limit
        let memory_buffer = 0.6;
        let result = ((available_memory_in_kb as f64 * memory_buffer / avg_size_of_block_kb as f64)
            / no_of_threads as f64)
            .floor() as usize;
        info!("Maximum number of blocks to pull in one call per thread is: {}", result);
        result
    }

    fn get_available_memory_in_kb(&self) -> u64 {
        let mut system = System::new_all();
        system.refresh_all();
        let available_memory_kb = system.available_memory();
        info!("System has {}MB Available memory", available_memory_kb / 1000);
        available_memory_kb
    }

    fn estimate_size_of_block_in_kb(&self) -> u64 {
        250000
    }
}

struct SolanaBlock {}
