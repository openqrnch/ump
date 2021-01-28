use std::fmt;

/// Module-specific error codes.
#[derive(Debug)]
pub enum Error<E> {
  /// The server object has shut down.  This happens when clients:
  /// - attempt to send messages to a server that has been deallocated.
  /// - have their requests dropped from the serrver's queue because the
  ///   server itself was deallocated.
  ServerDisappeared,

  /// The message was delivered to the server, but the reply context was
  /// released before sending back a reply.
  NoReply,

  /// Application-specific error.
  /// The `E` type is typically declared as the third generic parameter to
  /// [`channel`](crate::channel()).
  App(E)
}

impl<E> Error<E> {
  pub fn into_apperr(self) -> Option<E> {
    match self {
      Error::App(e) => Some(e),
      _ => None
    }
  }
  pub fn unwrap_apperr(self) -> E {
    match self {
      Error::App(e) => e,
      _ => panic!("Not an Error::App")
    }
  }
}

impl<E: fmt::Debug> std::error::Error for Error<E> {}

impl<E> From<crate::rctx::Error<E>> for Error<E> {
  fn from(err: crate::rctx::Error<E>) -> Self {
    match err {
      crate::rctx::Error::Aborted => Error::ServerDisappeared,
      crate::rctx::Error::NoReply => Error::NoReply,
      crate::rctx::Error::App(e) => Error::App(e)
    }
  }
}

impl<E: fmt::Debug> fmt::Display for Error<E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match &*self {
      Error::ServerDisappeared => write!(f, "Server disappeared"),
      Error::NoReply => write!(f, "Server didn't reply"),
      Error::App(err) => write!(f, "Application error; {:?}", err)
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
