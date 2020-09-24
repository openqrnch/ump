//! Allow a thread/task, crossing sync/async boundaries in either direction, to
//! deliver an expected piece of data to another thread/task, with
//! notification.
//!
//! These are simple channels used to deliver data from one endpoint to
//! another, where the receiver will block until data has been delivered.

mod err;
mod inner;

pub mod public;

pub(crate) use err::Error;
pub(crate) use inner::InnerReplyContext;

pub use public::ReplyContext;

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
