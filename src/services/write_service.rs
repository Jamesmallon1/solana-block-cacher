use crate::model::solana_block::{BlockBatch, SerializedSolanaBlock};
use crate::utilities::priority_queue::PriorityQueue;
use log::{error, info};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use std::thread;

/// A service responsible for writing Solana block batches to a file.
///
/// The `WriteService` struct manages a queue of block batches (`BlockBatch`) and writes them to a specified output file.
/// It uses a separate thread to handle the writing process, ensuring that the main application thread remains unblocked.
///
/// # Fields
///
/// - `write_queue`: An `Arc` wrapped `PriorityQueue` that stores the block batches to be written to the file.
///
/// # Examples
///
/// ```
/// use crate::model::solana_block::BlockBatch;
/// use crate::utilities::priority_queue::PriorityQueue;
/// use crate::WriteService;
/// use std::sync::Arc;
///
/// let write_queue = Arc::new(PriorityQueue::new());
/// let write_service = WriteService::new(write_queue);
/// ```
pub struct WriteService {
    write_queue: Arc<PriorityQueue<BlockBatch>>,
}

impl WriteService {
    /// Creates a new instance of `WriteService`.
    ///
    /// This method initializes the `WriteService` with a shared `PriorityQueue` for block batches.
    /// The queue is expected to be shared with other parts of the application that produce block batches.
    ///
    /// # Arguments
    ///
    /// * `write_queue`: An `Arc` wrapped `PriorityQueue<BlockBatch>` from which the service will consume block batches.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `WriteService`.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::model::solana_block::BlockBatch;
    /// use crate::utilities::priority_queue::PriorityQueue;
    /// use crate::WriteService;
    /// use std::sync::Arc;
    ///
    /// let write_queue = Arc::new(PriorityQueue::new());
    /// let write_service = WriteService::new(write_queue);
    /// ```
    pub fn new(write_queue: Arc<PriorityQueue<BlockBatch>>) -> Self {
        WriteService { write_queue }
    }

    /// Initializes the service to start writing block batches to the specified output file.
    ///
    /// This method spawns a new thread that continuously monitors the `write_queue`. When block batches
    /// are available, it writes them to the specified output file in order of their sequence numbers.
    ///
    /// # Arguments
    ///
    /// * `output_file`: A `String` specifying the path to the output file where block batches will be written.
    ///
    /// # Behavior
    ///
    /// - The method creates and runs a new thread that waits for block batches to appear in the queue.
    /// - It writes each block batch to the specified file, ensuring that they are written in the correct sequence.
    /// - If an error occurs during file writing, it logs the error but continues to process subsequent batches.
    ///
    /// # Side Effects
    ///
    /// - Creates a new thread for handling file writing.
    /// - Opens and writes to the output file specified.
    ///
    /// # Panics
    ///
    /// This method panics if it fails to open the output file.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::model::solana_block::BlockBatch;
    /// use crate::utilities::priority_queue::PriorityQueue;
    /// use crate::WriteService;
    /// use std::sync::Arc;
    ///
    /// let write_queue = Arc::new(PriorityQueue::new());
    /// let write_service = WriteService::new(write_queue);
    /// write_service.initialize(String::from("path/to/output_file.txt"));
    /// ```
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
