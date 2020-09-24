use std::sync::Weak;

use sigq::Queue as NotifyQueue;

use crate::err::Error;
use crate::rctx::InnerReplyContext;
use crate::srvq::ServerQueueNode;

/// Representation of a clonable client object.
///
/// Each instantiation of a `Client` object is itself an isolated client with
/// regards to the server context.  By cloning a client a new independent
/// client is created.  (Independent here meaning that it is still tied to the
/// same server object, but the new client can be passed to a separate thread
/// and can independently make calls to the server).
pub struct Client<S, R> {
  /// Weak reference to server queue.
  ///
  /// The server context holds the only strong reference to the queue.  This
  /// allows the clients to detect when the server has terminated.
  pub(crate) srvq: Weak<NotifyQueue<ServerQueueNode<S, R>>>
}

impl<S, R> Client<S, R>
where
  R: 'static + Send
{
  /// Send a message to the server, wait for a reply, and return the reply.
  ///
  /// A complete round-trip (the message is delivered to the server, and the
  /// server sends a reply) must complete before this function returns
  /// success.
  ///
  /// This method is _currently_ reentrant: It is safe to use share a single
  /// `Client` among multiple threads.  _This may change in the future_; it's
  /// best not to rely on this.  The recommended way to send messages to a
  /// server from multiple threads is to clone the `Client` and move the clones
  /// to the separate threads.
  ///
  /// # Return
  /// On success the function will return `Ok(msg)`.
  ///
  /// If the linked server object has been released
  /// `Err(Error:ServerDisappeared)` will be returned.
  ///
  /// If the server never replied to the message and the reply context was
  /// dropped `Err(Error::Aborted)` will be returned.
  pub fn send(&self, out: S) -> Result<R, Error> {
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

    let reply = rctx.get()?;
    Ok(reply)
  }

  /// Same as [`send`](#method.send) but for use in `async` contexts.
  pub async fn asend(&self, out: S) -> Result<R, Error> {
    let srvq = match self.srvq.upgrade() {
      Some(srvq) => srvq,
      None => return Err(Error::ServerDisappeared)
    };

    let rctx = InnerReplyContext::new();

    srvq.push(ServerQueueNode {
      msg: out,
      reply: rctx.clone()
    });

    let result = rctx.aget().await?;

    Ok(result)
  }
}


/// When a client is cloned then create an entirely new client.  It will be
/// linked to the same server, but in all other respects the clone is a
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
