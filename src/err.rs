use std::fmt;

/// Module-specific error codes.
#[derive(Debug)]
pub enum Error {
  /// The server object has shut down.  This happens when clients:
  /// - attempt to send messages to a server that has been deallocated.
  /// - have their requests dropped from the serrver's queue because the
  ///   server itself was deallocated.
  ServerDisappeared,

  /// The message was delivered to the server, but the reply context was
  /// released before sending back a reply.
  NoReply
}

impl std::error::Error for Error {}

impl From<crate::rctx::Error> for Error {
  fn from(err: crate::rctx::Error) -> Self {
    match err {
      crate::rctx::Error::Aborted => Error::ServerDisappeared,
      crate::rctx::Error::NoReply => Error::NoReply
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match &*self {
      Error::ServerDisappeared => write!(f, "Server disappeared"),
      Error::NoReply => write!(f, "Server didn't reply")
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
