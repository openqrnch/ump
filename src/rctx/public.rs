use crate::rctx::err::Error;
use crate::rctx::inner::State;
use crate::rctx::InnerReplyContext;

/// Public-facing sender part of the `ReplyContext` object.
///
/// This is safe to pass to applications which are meant to only be able to put
/// a value through the `ReplyContext` channel, but not extract the value from
/// it.
pub struct ReplyContext<I> {
  inner: InnerReplyContext<I>,
  did_handover: bool
}

impl<I: 'static + Send> ReplyContext<I> {
  /// Send a reply back to originating client.
  ///
  /// # Semantics
  /// This call is safe to make after the server context has been released.
  pub fn reply(mut self, data: I) -> Result<(), Error> {
    self.inner.put(data);

    self.did_handover = true;

    Ok(())
  }
}

impl<I> Drop for ReplyContext<I> {
  /// If the reply context is dropped while still waiting for a reply then
  /// report back to the caller that it should expect no reply.
  fn drop(&mut self) {
    if self.did_handover == false {
      let mut do_signal: bool = false;
      let mut mg = self.inner.data.lock().unwrap();
      match *mg {
        State::Waiting => {
          *mg = State::NoReply;
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

impl<I> From<InnerReplyContext<I>> for ReplyContext<I> {
  /// Transform an internal reply context into a public one and change the
  /// state from Queued to Waiting to signal that the node has left the
  /// queue.
  fn from(inner: InnerReplyContext<I>) -> Self {
    // Switch state from "Queued" to "Waiting", to mark that the reply context
    // has been "picked up".
    let mut mg = inner.data.lock().unwrap();
    match *mg {
      State::Queued => {
        *mg = State::Waiting;
        drop(mg);
      }
      _ => {
        // Should never happen
        drop(mg);
        panic!("Unexpected node state.");
      }
    }

    ReplyContext {
      inner: inner.clone(),
      did_handover: false
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
