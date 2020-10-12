use std::fmt;

/// Module-specific error codes.
#[derive(Debug)]
pub enum Error {
  /// The reply was aborted.
  Aborted,

  /// The public [`ReplyContext`] object is required to reply with a value.
  /// If it does not the endpoint waiting to receive a value will abort and
  /// return this error.
  NoReply
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match &*self {
      Error::Aborted => write!(f, "Aborted call"),
      Error::NoReply => write!(f, "Application failed to reply")
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
