/// Ring buffer for the USB → BLE receipt pipeline.
///
/// The USB task pushes complete receipt byte vectors into the queue.
/// The main loop pops them and sends via BLE GATT notifications.
///
/// Uses a simple Mutex + VecDeque since we have std on ESP-IDF.
/// For a lock-free alternative, swap to `heapless::spsc::Queue`.

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

/// Thread-safe receipt queue with blocking receive.
pub struct ReceiptBuffer {
    queue: Mutex<VecDeque<Vec<u8>>>,
    /// Signals the consumer when a new receipt is available.
    ready: Condvar,
    capacity: usize,
}

impl ReceiptBuffer {
    /// Create a new buffer with the given max capacity.
    /// Receipts pushed beyond capacity are dropped (logged).
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            ready: Condvar::new(),
            capacity,
        }
    }

    /// Push a complete receipt into the buffer.
    /// Returns false if the buffer is full (receipt dropped).
    pub fn push(&self, receipt: Vec<u8>) -> bool {
        let mut q = self.queue.lock().unwrap();
        if q.len() >= self.capacity {
            return false;
        }
        q.push_back(receipt);
        self.ready.notify_one();
        true
    }

    /// Block until a receipt is available, then return it.
    pub fn recv(&self) -> Vec<u8> {
        let mut q = self.queue.lock().unwrap();
        loop {
            if let Some(receipt) = q.pop_front() {
                return receipt;
            }
            q = self.ready.wait(q).unwrap();
        }
    }

    /// Non-blocking try_recv.
    pub fn try_recv(&self) -> Option<Vec<u8>> {
        self.queue.lock().unwrap().pop_front()
    }
}
