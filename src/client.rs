use std::sync::Weak;

use crate::err::Error;
use crate::nq::NotifyQueue;
use crate::rctx::{self, InnerReplyContext};
use crate::srvq::ServerQueueNode;

/// Representation of a client object.
///
/// Each instantiation of a `Client` object is itself an isolated client with
/// regards to the server context.  By cloning a client a new independent
/// client is created.  (Independent here meaning that it is still tied to the
/// same server object, but it the new client can be passed to a separate
/// thread and can independently make calls to the server).
pub struct Client<S, R> {
  /// Weak reference to server queue.
  ///
  /// The server context holds the only strong reference to the queue.  This
  /// allows the clients to detect when the server has terminated.
  pub(crate) srvq: Weak<NotifyQueue<ServerQueueNode<S, R>>>
}

impl<S, R> Client<S, R> {
  /// Send a message to the server and expect a reply.
  ///
  /// # Invariants
  /// - A complete send-and-fetch-reply must complete before this function
  ///   returns success.
  /// - Because this function takes a mutable reference to self it can not be
  ///   called reentrantly.
  pub fn send(&mut self, out: S) -> Result<R, Error> {
    // # Hand over message to server #########################################

    // Make sure the server still lives; Weak -> Arc
    let srvq = match self.srvq.upgrade() {
      Some(srvq) => srvq,
      None => return Err(Error::ServerDisappeared)
    };

    // Create a per-call reply context.
    // This context could be created when the Client object is being created
    // and stored in the context, and thus be reused for reach client call.
    // One side-effect is that some of the state semantics becomes more
    // complicated.
    // The first checkin in the repo has such an implementation, but it seems
    // to have some more corner cases that aren't properly handled.
    let rctx = InnerReplyContext::new();

    // Lock the server queue
    let mut q = srvq.lockq();

    // Put reply into queue
    q.push_back(ServerQueueNode {
      msg: out,
      reply: rctx.clone()
    });

    // unlock server queue asap
    drop(q);

    // notify server that a message is available for pickup
    srvq.notify();

    // # Wait for a reply ####################################################

    // Lock reply data
    let mut mg = rctx.data.lock().unwrap();

    // Wait for reply
    let msg = loop {
      match &*mg {
        rctx::State::Waiting => {
          // Wait until server reports back that there's data
          mg = rctx.signal.wait(mg).unwrap();
          continue;
        }
        rctx::State::Message(_msg) => {
          // Revert back to Wairing state and return the message.
          if let rctx::State::Message(msg) =
            std::mem::replace(&mut *mg, rctx::State::Finalized)
          {
            break msg;
          } else {
            // We're *really* in trouble if this happens ..
            let s = String::from("Not State::Message()");
            return Err(Error::BadInternalState(s));
          }
        }
        rctx::State::Finalized => {
          let s = String::from("Finalized instead of State::Message()");
          return Err(Error::BadInternalState(s));
        }
        rctx::State::Aborted => {
          return Err(Error::Aborted);
        }
      }
    };
    drop(mg);

    Ok(msg)
  }
}


/// When a client is cloned then create an entirely new client.  It will be
/// tied to the same server, but in all other respects the clone is a
/// completely new client.
///
/// This means that a cloned client can be passed to a new thread/task and make
/// new independent calls to the server without any risk of collision between
/// clone and the original client object.
impl<S, R> Clone for Client<S, R> {
  fn clone(&self) -> Self {
    Client {
      srvq: Weak::clone(&self.srvq)
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
