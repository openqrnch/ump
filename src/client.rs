use std::sync::Weak;

use sigq::Queue as NotifyQueue;

use crate::err::Error;
use crate::rctx::InnerReplyContext;
use crate::server::ServerQueueNode;

/// Representation of a clonable client object.
///
/// Each instantiation of a `Client` object is itself an isolated client with
/// regards to the server context.  By cloning a client a new independent
/// client is created.  ("Independent" here meaning that it is still tied to
/// the same server object, but the new client can be passed to a separate
/// thread and can independently make calls to the server).
pub struct Client<S, R, E> {
  /// Weak reference to server queue.
  ///
  /// The server context holds the only strong reference to the queue.  This
  /// allows the clients to detect when the server has terminated.
  pub(crate) srvq: Weak<NotifyQueue<ServerQueueNode<S, R, E>>>
}

impl<S, R, E> Client<S, R, E>
where
  R: 'static + Send,
  E: 'static + Send
{
  /// Send a message to the server, wait for a reply, and return the reply.
  ///
  /// A complete round-trip (the message is delivered to the server, and the
  /// server sends a reply) must complete before this function returns
  /// success.
  ///
  /// This method is _currently_ reentrant: It is safe to use share a single
  /// `Client` among multiple threads.  _This may change in the future_; it's
  /// recommended to not rely on this.  The recommended way to send messages to
  /// a server from multiple threads is to clone the `Client` and assign one
  /// separate `Client` to each thread.
  ///
  /// # Return
  /// On success the function will return `Ok(msg)`.
  ///
  /// If the linked server object has been released, or is released while the
  /// message is in the server's queue, `Err(Error:ServerDisappeared)` will be
  /// returned.
  ///
  /// If the server never replied to the message and the reply context was
  /// dropped `Err(Error::NoReply)` will be returned.
  ///
  /// If an application specific error occurs it will be returned as a
  /// `Err(Error::App(E))`, where `E` is the error type used when creating the
  /// [`channel`](crate::channel).
  pub fn send(&self, out: S) -> Result<R, Error<E>> {
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
    // The central repo has such an implementation checked in, but it seems to
    // have some more corner cases that aren't properly handled.
    let rctx = InnerReplyContext::new();

    srvq.push(ServerQueueNode {
      msg: out,
      reply: rctx.clone()
    });

    // Drop the strong server queue ref immediately so it's not held as a
    // strong ref while we're waiting for a reply.
    drop(srvq);

    let reply = rctx.get()?;
    Ok(reply)
  }

  /// Same as [`Client::send()`] but for use in `async` contexts.
  pub async fn asend(&self, out: S) -> Result<R, Error<E>> {
    let srvq = match self.srvq.upgrade() {
      Some(srvq) => srvq,
      None => return Err(Error::ServerDisappeared)
    };

    let rctx = InnerReplyContext::new();

    srvq.push(ServerQueueNode {
      msg: out,
      reply: rctx.clone()
    });

    // Drop the strong server queue ref immediately so it's not held as a
    // strong ref while we're waiting for a reply.
    drop(srvq);

    let result = rctx.aget().await?;

    Ok(result)
  }
}


impl<S, R, E> Clone for Client<S, R, E> {
  /// Clone a client.
  ///
  /// When a client is cloned the new object will be linked to the same server,
  /// but in all other respects the clone is a completely independent client.
  ///
  /// This means that a cloned client can be passed to a new thread/task and
  /// make new independent calls to the server without any risk of collision
  /// between clone and the original client object.
  fn clone(&self) -> Self {
    Client {
      srvq: Weak::clone(&self.srvq)
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
