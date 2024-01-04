use crate::model::solana_block;
use crate::model::solana_block::{BlockBatch, Reverse};
use crate::utilities::priority_queue::Queue;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
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
pub struct WriteService<P: for<'a> Queue<'a, Reverse<BlockBatch>>> {
    write_queue: Arc<Mutex<P>>,
    condvar: Arc<Condvar>,
}

impl<P: for<'a> Queue<'a, Reverse<BlockBatch>> + Send + 'static> WriteService<P> {
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
    pub fn new(write_queue: Arc<Mutex<P>>, condvar: Arc<Condvar>) -> Self {
        WriteService { write_queue, condvar }
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
    pub fn initialize(&mut self, output_file: &str, slot_range: u64) {
        let queue_clone = self.write_queue.clone();
        let condvar_clone = self.condvar.clone();
        let progress_bar = self.configure_progress_bar(slot_range);
        let mut file = self.open_file(output_file, true);
        self.write_character_to_own_line(&mut file, '[');
        thread::spawn(move || {
            let mut next_sequence_id = 1_u64;
            {
                loop {
                    wait_for_data(queue_clone.clone(), &condvar_clone.clone(), next_sequence_id);
                    let mut queue = queue_clone.lock().unwrap();
                    debug!("Checking to see if there are any blocks to write to file");
                    while queue.peek().is_some() && queue.peek().unwrap().0.sequence_number == next_sequence_id {
                        WriteService::write_batch_to_file(
                            &mut queue,
                            slot_range,
                            &mut file,
                            &progress_bar,
                            &mut next_sequence_id,
                        )
                    }
                }
            }
        });
    }

    /// Completes the JSON file writing process.
    ///
    /// This method finalizes the JSON file that has been written to by the `WriteService`.
    /// It seeks to the end of the file and truncates the last byte (typically a comma) to prepare for the closing character.
    /// Then, it writes the closing character (']') on its own line to properly close the JSON array structure.
    ///
    /// # Arguments
    ///
    /// * `output_file`: A string slice reference (`&str`) representing the path of the output file.
    ///
    /// # Behavior
    ///
    /// - The method seeks to the end of the specified output file.
    /// - If the file is not empty, it truncates the last byte, usually to remove a trailing comma.
    /// - Writes the closing JSON array character (']') on a new line.
    ///
    /// # Side Effects
    ///
    /// - Modifies the content of the output file by truncating its last byte and appending a character.
    ///
    /// # Errors
    ///
    /// - The method logs an error and does not panic if it encounters issues while seeking to the end of the file,
    ///   truncating the file, or writing to the file.
    ///
    /// # Panics
    ///
    /// - This method panics if it fails to open the output file or if file operations like seeking, truncating, or writing encounter errors.
    ///
    /// # Examples
    ///
    /// ```
    /// let write_service = WriteService::new(...); // Initialization of WriteService
    /// write_service.complete_json_file("path/to/output_file.json");
    /// ```
    ///
    /// After calling this method, the JSON file at the specified path will be properly closed and finalized.
    pub fn complete_json_file(&self, output_file: &str) {
        info!("Completing JSON file: {}", output_file);
        let mut file = self.open_file(output_file, false);
        let file_length = file.seek(SeekFrom::End(0)).expect("Unable to seek to end of file");
        if file_length > 0 {
            // truncate file by 1 byte
            file.set_len(file_length - 2).expect("Unable to truncate the file");
        }

        self.write_character_to_own_line(&mut file, '\n');
        self.write_character_to_own_line(&mut file, ']');
    }

    fn write_batch_to_file(
        wq: &mut MutexGuard<P>,
        slot_range: u64,
        file: &mut File,
        progress_bar: &ProgressBar,
        next_sequence_id: &mut u64,
    ) {
        let block_batch = wq.pop().unwrap();
        debug!(
            "Attempting to write block batch {} to file",
            block_batch.0.sequence_number
        );
        for block in block_batch.0.batch {
            if let Err(e) = writeln!(file, "{},", block.data) {
                error!("Could not write block on slot {} to file: {}", block.slot_number, e);
            }
        }
        debug!("Block batch {} written to file", block_batch.0.sequence_number);
        progress_bar.inc(solana_block::BATCH_SIZE);
        if progress_bar.position() > slot_range - solana_block::BATCH_SIZE {
            progress_bar.finish_and_clear();
        }
        *next_sequence_id += 1;
    }

    fn configure_progress_bar(&self, slot_range: u64) -> ProgressBar {
        let progress_bar = ProgressBar::new(slot_range);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{prefix:.bold.dim} [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
                .expect("Template style for progress bar is invalid.")
                .progress_chars("##-"),
        );

        progress_bar
    }

    fn open_file(&self, output_file: &str, clear_file_on_open: bool) -> File {
        let file = OpenOptions::new().append(true).create(true).open(output_file).expect("Unable to open file");
        let metadata = file.metadata().expect("Unable to get file metadata");
        if metadata.len() > 0 && clear_file_on_open {
            debug!("File at path: {} is not empty, truncating now..", &output_file);
            file.set_len(0).expect("Unable to truncate file");
        }

        file
    }

    fn write_character_to_own_line(&self, file: &mut File, character: char) {
        if let Err(e) = writeln!(file, "{}", character) {
            error!("Could not write character {} to file: {}", character, e);
        }
    }
}

// circumvent lifetime issues using higher ranked trait bounds
fn wait_for_data<PQ: for<'a> Queue<'a, Reverse<BlockBatch>> + 'static>(
    pq: Arc<Mutex<PQ>>,
    condvar: &Condvar,
    next_sequence_id: u64,
) {
    let mut queue = pq.lock().unwrap();
    while queue.peek().is_none()
        || (queue.peek().is_some() && queue.peek().unwrap().0.sequence_number != next_sequence_id)
    {
        queue = condvar.wait(queue).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utilities::priority_queue::PriorityQueue;
    use std::fs;
    use std::io::Read;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    // helpers
    fn create_non_empty_file(file_path: &str) {
        let mut file = File::create(file_path).expect("Failed to create file");
        writeln!(file, "Sample data").expect("Failed to write to file");
    }

    #[test]
    fn test_write_service_new() {
        let write_queue = Arc::new(Mutex::new(PriorityQueue::new()));
        let condvar = Arc::new(Condvar::new());

        // Clone the Arcs to compare them later
        let write_queue_clone = Arc::clone(&write_queue);
        let condvar_clone = Arc::clone(&condvar);

        let write_service = WriteService::new(write_queue, condvar);

        assert!(Arc::ptr_eq(&write_service.write_queue, &write_queue_clone));
        assert!(write_service.write_queue.lock().is_ok());

        assert!(Arc::ptr_eq(&write_service.condvar, &condvar_clone));
    }

    #[test]
    fn test_creates_new_file() {
        let file_path = "test_output_new.txt";
        let _ = fs::remove_file(file_path);

        let write_queue = Arc::new(Mutex::new(PriorityQueue::new()));
        let condvar = Arc::new(Condvar::new());
        let write_service = WriteService::new(write_queue, condvar);
        write_service.open_file(file_path, true);

        assert!(Path::new(file_path).exists(), "File should be created");
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_does_not_truncate_non_empty_file() {
        let file_path = "test_output_non_truncate.txt";
        create_non_empty_file(file_path);

        let write_queue = Arc::new(Mutex::new(PriorityQueue::new()));
        let condvar = Arc::new(Condvar::new());
        let write_service = WriteService::new(write_queue, condvar);
        write_service.open_file(file_path, false);

        let metadata = fs::metadata(file_path).expect("Failed to read metadata");
        assert!(metadata.len() > 0);
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_truncates_non_empty_file() {
        let file_path = "test_output_truncate.txt";
        create_non_empty_file(file_path);

        let write_queue = Arc::new(Mutex::new(PriorityQueue::new()));
        let condvar = Arc::new(Condvar::new());
        let write_service = WriteService::new(write_queue, condvar);
        write_service.open_file(file_path, true);

        let metadata = fs::metadata(file_path).expect("Failed to read metadata");
        assert_eq!(metadata.len(), 0, "File should be truncated");
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_opens_empty_file_without_modification() {
        let file_path = "test_output_empty.txt";
        let _ = File::create(file_path).expect("Failed to create file");

        let write_queue = Arc::new(Mutex::new(PriorityQueue::new()));
        let condvar = Arc::new(Condvar::new());
        let write_service = WriteService::new(write_queue, condvar);
        write_service.open_file(file_path, true);

        let metadata = fs::metadata(file_path).expect("Failed to read metadata");
        assert_eq!(metadata.len(), 0, "File should remain empty");
        let _ = fs::remove_file(file_path);
    }

    #[test]
    #[should_panic(expected = "Unable to open file")]
    fn test_error_handling_file_open() {
        let file_path = "/invalid/path/test_output.txt";

        let write_queue = Arc::new(Mutex::new(PriorityQueue::new()));
        let condvar = Arc::new(Condvar::new());
        let write_service = WriteService::new(write_queue, condvar);

        write_service.open_file(file_path, true);
    }

    #[test]
    fn test_write_character_to_own_line() {
        let write_queue = Arc::new(Mutex::new(PriorityQueue::new()));
        let condvar = Arc::new(Condvar::new());
        let write_service = WriteService::new(write_queue, condvar);
        let path = "temp_test_file.txt";

        let mut file = File::create(path).unwrap();

        write_service.write_character_to_own_line(&mut file, 'a');

        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert_eq!(contents, "a\n");

        fs::remove_file(path).unwrap();
    }
}
