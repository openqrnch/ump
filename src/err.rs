use std::fmt;

/// Module-specific error codes.
#[derive(Debug)]
pub enum Error {
  ServerDisappeared,
  Aborted,
  BadInternalState(String),
  BadFormat(String)
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match &*self {
      Error::ServerDisappeared => write!(f, "Server disappeared"),
      Error::Aborted => write!(f, "Aborted call"),
      Error::BadInternalState(s) => write!(f, "Internal state error; {}", s),
      Error::BadFormat(s) => write!(f, "Bad format error; {}", s)
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
