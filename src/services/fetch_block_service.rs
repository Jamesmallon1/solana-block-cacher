use crate::model::solana_block;
use crate::model::solana_block::{BlockBatch, Reverse, SerializedSolanaBlock};
use crate::utilities::priority_queue::PriorityQueue;
use crate::utilities::rate_limiter::RateLimiter;
use crate::utilities::threading::ThreadPool;
use log::{error, info};
use solana_client::rpc_client::RpcClient;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

pub struct FetchBlockService {
    write_queue: Arc<Mutex<PriorityQueue<Reverse<BlockBatch>>>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    thread_pool: ThreadPool,
    condvar: Arc<Condvar>,
}

impl FetchBlockService {
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

    pub fn fetch_blocks(&mut self, from_slot: u64, to_slot: u64, rpc_url: &str) {
        // calculate slots per thread and initialize thread completion counter
        let no_of_threads = self.thread_pool.get_number_of_workers();
        let total_slots = to_slot - from_slot + 1;
        let slots_per_thread = total_slots / no_of_threads as u64;
        let completed_count = Arc::new(Mutex::new(0));

        for i in 0..no_of_threads {
            // calculate the range of blocks for each thread
            let start_slot = from_slot + i as u64 * slots_per_thread;
            let end_slot = if i == no_of_threads - 1 {
                to_slot
            } else {
                start_slot + slots_per_thread - 1
            };

            // clone necessary variables prior to movement
            let completed_clone = completed_count.clone();
            let rpc_str = rpc_url.to_string();
            let global_start_slot = from_slot.clone();
            let rl_clone = self.rate_limiter.clone();
            let queue_clone = self.write_queue.clone();
            let condvar_clone = self.condvar.clone();

            // create the closure
            let closure = move || {
                let rpc_client = RpcClient::new(rpc_str);
                let number_of_slots = end_slot - start_slot;
                let number_of_block_batches = if number_of_slots <= solana_block::BATCH_SIZE {
                    1_u64
                } else {
                    (number_of_slots as f64 / solana_block::BATCH_SIZE as f64).ceil() as u64
                };
                for i in 1..=number_of_block_batches {
                    // calculate the batch configuration
                    let batch_start_slot = start_slot + (solana_block::BATCH_SIZE * (i - 1));
                    let batch_end_slot = if i == number_of_block_batches - 1 {
                        end_slot
                    } else {
                        batch_start_slot + solana_block::BATCH_SIZE - 1
                    };
                    let seq_id = (((batch_start_slot as f64 - global_start_slot as f64)
                        / solana_block::BATCH_SIZE as f64)
                        .floor()
                        + 1.0) as u64;
                    let mut block_list: Vec<SerializedSolanaBlock> = vec![];

                    // populate block_list
                    for slot_number in batch_start_slot..=batch_end_slot {
                        while rl_clone.lock().unwrap().should_wait() {
                            thread::sleep(Duration::from_millis(10));
                        }
                        let block_result = rpc_client.get_block(slot_number + 1);
                        match block_result {
                            Ok(block) => {
                                let serialized_data_result = serde_json::to_string(&block);
                                match serialized_data_result {
                                    Ok(data) => {
                                        let solana_block = SerializedSolanaBlock {
                                            slot_number: block.parent_slot as u64,
                                            data,
                                        };
                                        block_list.push(solana_block);
                                    }
                                    Err(err) => {
                                        error!(
                                            "An error occurred serializing a Solana block: {}, Error: {}",
                                            slot_number, err
                                        );
                                    }
                                }
                            }
                            Err(err) => {
                                error!("Could not retrieve block {} due to error: {}", slot_number, err);
                            }
                        }
                    }

                    info!(
                        "Dispatching block batch {}-{} to be written to file.",
                        batch_start_slot, batch_end_slot
                    );
                    // ship batch to write thread
                    let batch = BlockBatch {
                        sequence_number: seq_id,
                        batch: block_list,
                    };
                    queue_clone.lock().unwrap().push(Reverse(batch));
                    condvar_clone.notify_one();
                }

                let mut completed = completed_clone.lock().unwrap();
                *completed += 1;
            };

            // execute the closure in the thread pool
            self.thread_pool.execute(closure);
        }

        info!("Block caching has started");
        loop {
            let completed = completed_count.lock().unwrap();
            if *completed == no_of_threads as u32 {
                break;
            }
            drop(completed);
            thread::sleep(Duration::from_millis(100));
        }

        // cleanup
        self.thread_pool.destroy();
    }
}
