use std::sync::{Arc, Condvar, Mutex};

use crate::err::Error;

pub(crate) enum State<R> {
  /// Waiting for a reply
  Waiting,

  /// A message has been returned.
  Message(R),

  /// The client has picked up the reply.
  Finalized,

  /// Returned by the Drop handler if it detects that a reply hasn't been
  /// sent.
  Aborted
}

/// This is essentially the same as ReplyContext, but it does not implement the
/// `Drop` trait.  This is important because only the application exposed reply
/// context must implement `Drop`.
pub struct InnerReplyContext<R> {
  pub(crate) signal: Arc<Condvar>,
  pub(crate) data: Arc<Mutex<State<R>>>
}

impl<R> InnerReplyContext<R> {
  /// Create an inner reply context in idle state.
  ///
  /// This must not be exposed directly to the application.
  pub(crate) fn new() -> Self {
    let data_state = State::Waiting;
    InnerReplyContext {
      signal: Arc::new(Condvar::new()),
      data: Arc::new(Mutex::new(data_state))
    }
  }
}

impl<R> Clone for InnerReplyContext<R> {
  fn clone(&self) -> Self {
    InnerReplyContext {
      signal: Arc::clone(&self.signal),
      data: Arc::clone(&self.data)
    }
  }
}


/// Structure instantiated by server when a client has sent a message and is
/// waiting for a reply.
pub struct ReplyContext<R> {
  pub(crate) inner: InnerReplyContext<R>,
  did_reply: bool
}

impl<R> ReplyContext<R> {
  /// Consume the reply context and send a reply back to caller.
  ///
  /// Calling this method is mandatory.  The client will block until it has
  /// been signalled by this method.
  pub fn reply(mut self, data: R) -> Result<(), Error> {
    let mut mg = self.inner.data.lock().unwrap();
    *mg = State::Message(data);
    drop(mg);

    // Tell the client that it has a reply to pick up
    //eprintln!("Signal client that a reply is available!");
    self.inner.signal.notify_one();

    self.did_reply = true;

    Ok(())
  }
}

/// If the server hasn't explicitly replied when the reply context is dropped
/// then notify the client that the reply was aborted.  Applications should
/// never use this mechanism; it's explicitly an error.
impl<R> Drop for ReplyContext<R> {
  fn drop(&mut self) {
    if self.did_reply == false {
      //eprintln!("Warning: ReplyContext didn't reply!");
      let mut do_signal: bool = false;
      let mut mg = self.inner.data.lock().unwrap();
      match *mg {
        State::Waiting => {
          *mg = State::Aborted;
          do_signal = true;
        }
        _ => {}
      }
      drop(mg);
      if do_signal {
        self.inner.signal.notify_one();
      }
    }
  }
}

impl<R> From<InnerReplyContext<R>> for ReplyContext<R> {
  fn from(inner: InnerReplyContext<R>) -> Self {
    ReplyContext {
      inner: inner.clone(),
      did_reply: false
    }
  }
}

impl<R> From<&InnerReplyContext<R>> for ReplyContext<R> {
  fn from(inner: &InnerReplyContext<R>) -> Self {
    ReplyContext {
      inner: inner.clone(),
      did_reply: false
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
