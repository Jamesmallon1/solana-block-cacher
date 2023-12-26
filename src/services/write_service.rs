use crate::model::solana_block::{BlockBatch, SerializedSolanaBlock};
use crate::utilities::priority_queue::PriorityQueue;
use log::{error, info};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use std::thread;

pub struct WriteService {
    write_queue: Arc<PriorityQueue<BlockBatch>>,
}

impl WriteService {
    pub fn new(write_queue: Arc<PriorityQueue<BlockBatch>>) -> Self {
        WriteService { write_queue }
    }

    pub fn initialize(&self, output_file: String) {
        let mut queue_clone = self.write_queue.clone();
        thread::spawn(move || {
            let mut next_sequence_id = 1_u64;
            {
                loop {
                    queue_clone.wait_for_data();
                    let mut file =
                        OpenOptions::new().append(true).create(true).open(&output_file).expect("Unable to open file");
                    info!("Checking to see if there are any blocks to write to file");
                    while queue_clone.peek().is_some()
                        && queue_clone.peek().unwrap().sequence_number == next_sequence_id
                    {
                        let block_batch = queue_clone.pop().unwrap();
                        info!(
                            "Attempting to write block batch {} to file",
                            block_batch.sequence_number
                        );
                        for block in block_batch.batch {
                            if let Err(e) = writeln!(file, "{}", block.data) {
                                error!("Could not write block on slot {} to file: {}", block.slot_number, e);
                            }
                        }
                        info!("Block batch {} written to file", block_batch.sequence_number);
                        next_sequence_id += 1;
                    }
                }
            }
        });
    }
}
