use std::collections::BinaryHeap;
use std::sync::{Condvar, Mutex};

/// A generic priority queue implemented using a binary heap.
///
/// The `PriorityQueue` struct can store any type `T` that implements `Ord` and `PartialOrd`,
/// which are used to maintain the elements in a sorted order. The default behavior is that
/// the element with the highest value according to the `Ord` trait will be considered the highest priority.
pub struct PriorityQueue<T: Ord + PartialOrd> {
    heap: Mutex<BinaryHeap<T>>,
    condvar: Condvar,
}

impl<T: Ord + PartialOrd> PriorityQueue<T> {
    /// Creates a new, empty `PriorityQueue`.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    /// let mut pq = PriorityQueue::new();
    /// ```
    pub fn new() -> Self {
        PriorityQueue {
            heap: Mutex::new(BinaryHeap::new()),
            condvar: Condvar::new(),
        }
    }

    /// Adds an element to the priority queue.
    ///
    /// The position of the new element in the queue is determined by its priority relative to existing elements.
    ///
    /// # Arguments
    ///
    /// * `block` - The element to be added to the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    /// let mut pq = PriorityQueue::new();
    /// pq.push(5);
    /// ```
    pub fn push(&mut self, block: T) {
        let mut heap = self.heap.lock().unwrap();
        heap.push(block);
        self.condvar.notify_one();
    }

    /// Adds multiple elements to the priority queue.
    ///
    /// This method takes a vector of elements and adds each of them to the priority queue.
    /// The position of each new element in the queue is determined by its priority relative to existing elements.
    /// The elements are added one by one, maintaining the heap property of the priority queue after each insertion.
    ///
    /// # Arguments
    ///
    /// * `items` - A vector of elements to be added to the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    /// let mut pq = PriorityQueue::new();
    /// pq.push_many(vec![5, 3, 8]);
    /// assert_eq!(pq.pop(), Some(8)); // 8 has the highest priority
    /// ```
    pub fn push_many(&mut self, items: Vec<T>) {
        let mut heap = self.heap.lock().unwrap();
        for item in items {
            heap.push(item);
        }
        self.condvar.notify_all();
    }

    /// Removes and returns the highest priority element from the queue, if it is not empty.
    ///
    /// Returns `None` if the queue is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    /// let mut pq = PriorityQueue::new();
    /// pq.push(5);
    /// assert_eq!(pq.pop(), Some(5));
    /// assert_eq!(pq.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        let mut heap = self.heap.lock().unwrap();
        heap.pop()
    }

    /// Returns a reference to the highest priority element in the queue without removing it.
    ///
    /// This method returns `None` if the queue is empty. Otherwise, it returns `Some` with a reference to
    /// the element with the highest priority. The returned value is wrapped in an `Option` for safe handling.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    /// let mut pq = PriorityQueue::new();
    /// pq.push(5);
    /// pq.push(3);
    /// assert_eq!(pq.peek(), Some(&5));
    /// ```
    pub fn peek(&self) -> Option<&T> {
        let heap = self.heap.lock().unwrap();
        heap.peek()
    }

    /// Waits for data to be available in the priority queue.
    ///
    /// This method acquires a lock on the queue's internal storage and checks if it is empty.
    /// If the queue is empty, the method then waits on a condition variable until notified that new data has been pushed to the queue.
    ///
    /// # Behavior
    ///
    /// - The thread calling this method will block until other threads push data into the queue.
    /// - The lock on the queue is temporarily released while waiting, allowing other threads to push data.
    /// - Once notified, the thread wakes up and re-acquires the lock to safely access the queue.
    /// - If the queue is not empty when this method is called, it returns immediately without waiting.
    ///
    /// # Use Case
    ///
    /// This method is useful in scenarios where a thread needs to process data from the queue as soon as it becomes available, but also needs to avoid busy waiting or polling.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    ///
    /// // Assuming `pq` is a shared instance of PriorityQueue
    /// // accessible by multiple threads.
    ///
    /// // In a consumer thread
    /// loop {
    ///     pq.wait_for_data();
    ///     if let Some(data) = pq.pop() {
    ///         // Process the data
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - This method should be used cautiously to avoid deadlocks. Ensure that other threads are indeed pushing data to the queue.
    /// - The method is designed to handle spurious wakeups, but it's good practice for calling threads to re-check their conditions upon waking.
    pub fn wait_for_data(&self) {
        let mut heap = self.heap.lock().unwrap();
        while heap.is_empty() {
            heap = self.condvar.wait(heap).unwrap();
        }
    }
}
