use std::fmt;

/// Module-specific error codes.
#[derive(Debug)]
pub enum Error<E> {
  /// The reply was aborted.
  Aborted,

  /// The public [`ReplyContext`] object is required to reply with a value.
  /// If it does not the endpoint waiting to receive a value will abort and
  /// return this error.
  NoReply,

  /// An application-specific error occurred.
  App(E)
}

impl<E: fmt::Debug> std::error::Error for Error<E> {}

impl<E: fmt::Debug> fmt::Display for Error<E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match &*self {
      Error::Aborted => write!(f, "Aborted call"),
      Error::NoReply => write!(f, "Application failed to reply"),
      Error::App(err) => write!(f, "Application error; {:?}", err)
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
