use std::sync::Arc;

use sigq::Queue as NotifyQueue;

use crate::rctx::{InnerReplyContext, ReplyContext};

pub(crate) struct ServerQueueNode<S, R, E> {
  /// Raw message being sent from the client to the server.
  pub(crate) msg: S,

  /// Keep track of data needed to share reply data.
  pub(crate) reply: InnerReplyContext<R, E>
}

/// Representation of a server object.
///
/// Each instantiation of a [`Server`] object represents an end-point which
/// will be used to receive messages from connected [`Client`](crate::Client)
/// objects.
pub struct Server<S, R, E> {
  pub(crate) srvq: Arc<NotifyQueue<ServerQueueNode<S, R, E>>>
}

impl<S, R, E> Server<S, R, E>
where
  S: 'static + Send,
  R: 'static + Send,
  E: 'static + Send
{
  /// Block and wait, indefinitely, for an incoming message from a
  /// [`Client`](crate::Client).
  ///
  /// Returns the message sent by the client and a reply context.  The server
  /// must call [`ReplyContext::reply()`] on the reply context to pass a return
  /// value to the client.
  pub fn wait(&self) -> (S, ReplyContext<R, E>) {
    let node = self.srvq.pop();

    // Extract the data from the node
    let msg = node.msg;

    // Create an application reply context from the reply context in the queue
    // Implicitly changes state of the reply context from Queued to Waiting
    let rctx = ReplyContext::from(node.reply);

    (msg, rctx)
  }

  /// Same as [`Server::wait()`], but for use in an `async` context.
  pub async fn async_wait(&self) -> (S, ReplyContext<R, E>) {
    let node = self.srvq.apop().await;

    // Extract the data from the node
    let msg = node.msg;

    // Create an application reply context from the reply context in the queue
    // Implicitly changes state of the reply context from Queued to Waiting
    let rctx = ReplyContext::from(node.reply);

    (msg, rctx)
  }

  /// Returns a boolean indicating whether the queue is/was empty.  This isn't
  /// really useful unless used in very specific situations.  It mostly exists
  /// for test cases.
  pub fn was_empty(&self) -> bool {
    self.srvq.was_empty()
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
