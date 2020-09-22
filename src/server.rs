use std::sync::Arc;

use crate::nq::NotifyQueue;
use crate::rctx::ReplyContext;
use crate::srvq::ServerQueueNode;

/// Representation of a server object.
///
/// Each instantiation of a `Client` object is itself an isolated client with
/// regards to the server context.  By cloning a client a new independent
/// client is created.  (Independent here meaning that it is still tied to the
/// same server object, but it the new client can be passed to a separate
/// thread and can independently make calls to the server).
pub struct Server<S, R> {
  pub(crate) srvq: Arc<NotifyQueue<ServerQueueNode<S, R>>>
}

impl<S, R> Server<S, R> {
  /// Block and wait for an incoming message from a
  /// [`Client`](struct.Client.html).
  ///
  /// Returns the message sent by the client and a reply context.  The server
  /// must call `reply()` on the reply context to pass a return value to the
  /// client.
  pub fn wait(&self) -> (S, ReplyContext<R>) {
    // Lock server queue
    let mut mg = self.srvq.lockq();

    // Get the oldest node in the queue
    let node = loop {
      match mg.pop_front() {
        Some(node) => {
          break node;
        }
        None => {
          mg = self.srvq.signal.wait(mg).unwrap();
        }
      }
    };

    // Extract the data from the node
    let msg = node.msg;

    // Create an application reply context from the reply context in the queue
    let rctx = ReplyContext::from(node.reply);

    (msg, rctx)
  }

  pub fn is_empty(&self) -> bool {
    let mg = self.srvq.lockq();
    mg.is_empty()
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
