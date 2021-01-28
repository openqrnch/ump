use crate::rctx::err::Error;
use crate::rctx::inner::State;
use crate::rctx::InnerReplyContext;

/// Public-facing sender part of the `ReplyContext` object.
///
/// This is safe to pass to applications which are meant to only be able to put
/// a value through the `ReplyContext` channel, but not extract the value from
/// it.
pub struct ReplyContext<I, E> {
  inner: InnerReplyContext<I, E>,
  did_handover: bool
}

impl<I: 'static + Send, E> ReplyContext<I, E> {
  /// Send a reply back to originating client.
  ///
  /// # Example
  /// ```
  /// use std::thread;
  /// use ump::channel;
  ///
  /// fn main() {
  ///   let (server, client) = channel::<String, String, ()>();
  ///   let server_thread = thread::spawn(move || {
  ///     let (data, rctx) = server.wait();
  ///     let reply = format!("Hello, {}!", data);
  ///     rctx.reply(reply).unwrap();
  ///   });
  ///   let msg = String::from("Client");
  ///   let reply = client.send(String::from(msg)).unwrap();
  ///   assert_eq!(reply, "Hello, Client!");
  ///   server_thread.join().unwrap();
  /// }
  /// ```
  ///
  /// # Semantics
  /// This call is safe to make after the server context has been released.
  pub fn reply(mut self, data: I) -> Result<(), Error<E>> {
    self.inner.put(data);

    self.did_handover = true;

    Ok(())
  }

  /// Return an error to originating client.
  /// This will cause the calling client to return an error.  The error passed
  /// in the `err` parameter will be wrapped in a `Error::App(err)`.
  ///
  /// # Example
  ///
  /// ```
  /// use std::thread;
  /// use ump::{channel, Error};
  ///
  /// #[derive(Debug, PartialEq)]
  /// enum MyError {
  ///   SomeError(String)
  /// }
  ///
  /// fn main() {
  ///   let (server, client) = channel::<String, String, MyError>();
  ///   let server_thread = thread::spawn(move || {
  ///     let (_, rctx) = server.wait();
  ///     rctx.fail(MyError::SomeError("failed".to_string())).unwrap();
  ///   });
  ///   let msg = String::from("Client");
  ///   let reply = client.send(String::from(msg));
  ///   match reply {
  ///     Err(Error::App(MyError::SomeError(s))) => {
  ///       assert_eq!(s, "failed");
  ///     }
  ///     _ => {
  ///       panic!("Unexpected return value");
  ///     }
  ///   }
  ///   server_thread.join().unwrap();
  /// }
  /// ```
  ///
  /// # Semantics
  /// This call is safe to make after the server context has been released.
  pub fn fail(mut self, err: E) -> Result<(), Error<E>> {
    self.inner.fail(err);

    self.did_handover = true;

    Ok(())
  }
}

impl<I, E> Drop for ReplyContext<I, E> {
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

impl<I, E> From<InnerReplyContext<I, E>> for ReplyContext<I, E> {
  /// Transform an internal reply context into a public one and change the
  /// state from Queued to Waiting to signal that the node has left the
  /// queue.
  fn from(inner: InnerReplyContext<I, E>) -> Self {
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
