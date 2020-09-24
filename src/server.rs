use std::sync::Arc;

use sigq::Queue as NotifyQueue;

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

impl<S, R> Server<S, R>
where
  S: 'static + Send,
  R: 'static + Send
{
  /// Block and wait for an incoming message from a
  /// [`Client`](struct.Client.html).
  ///
  /// Returns the message sent by the client and a reply context.  The server
  /// must call `reply()` on the reply context to pass a return value to the
  /// client.
  pub fn wait(&self) -> (S, ReplyContext<R>) {
    let node = self.srvq.pop();

    // Extract the data from the node
    let msg = node.msg;

    // Create an application reply context from the reply context in the queue
    let rctx = ReplyContext::from(node.reply);

    (msg, rctx)
  }

  pub async fn async_wait(&self) -> (S, ReplyContext<R>) {
    let node = self.srvq.apop().await;

    // Extract the data from the node
    let msg = node.msg;

    // Create an application reply context from the reply context in the queue
    let rctx = ReplyContext::from(node.reply);

    (msg, rctx)
  }

  pub fn was_empty(&self) -> bool {
    self.srvq.was_empty()
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
