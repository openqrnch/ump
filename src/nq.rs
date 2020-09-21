use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, MutexGuard};

/// Multithreaded queue with notification.
pub(crate) struct NotifyQueue<T> {
  pub(crate) signal: Condvar,
  pub(crate) queue: Mutex<VecDeque<T>>
}

impl<T> NotifyQueue<T> {
  pub(crate) fn new() -> Self {
    NotifyQueue {
      signal: Condvar::new(),
      queue: Mutex::new(VecDeque::new())
    }
  }
  pub(crate) fn notify(&self) {
    self.signal.notify_one();
  }

  pub(crate) fn lockq(&self) -> MutexGuard<VecDeque<T>> {
    self.queue.lock().unwrap()
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
