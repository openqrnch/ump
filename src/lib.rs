//! Micro Message Pass (ump) is a library for passing messages between
//! thread/tasks.  It has some similarities with the common mpsc channel
//! libraries, but with the most notable difference that each time a client
//! sends a message the server must send back a reply.
//!
//! The primary purpose of ump is to create simple RPC like designs, but
//! between threads/tasks within a process rather than between processes over
//! networks.
//!
//! # High-level usage overview
//! An application calls [`channel`] to create a linked pair of a [`Server`]
//! and a [`Client`].
//!
//! The server calls [`Server::wait()`]/[`Server::async_wait()`], which
//! blocks and waits for an incoming message from a client.
//!
//! A client, in a separate thread or task, calls
//! [`Client::send()`]/[`Client::asend()`] to send a message to the server.
//!
//! The server's wait call returns two objects:  The message sent by the
//! client, and a [`ReplyContext`].
//!
//! After processing its application-defined message, the server *must* call
//! the [`ReplyContext::reply()`] on the returned reply context object to
//! return a reply message to the client.
//!
//! Typically the server calls wait again to wait for next message from a
//! client.
//!
//! The client receives the reply from the server and processes it.
//!
//! # Example
//! ```
//! use std::thread;
//!
//! use ump::channel;
//!
//! fn main() {
//!  let (server, client) = channel::<String, String, ()>();
//!
//!  let server_thread = thread::spawn(move || {
//!    // Wait for data to arrive from a client
//!    println!("Server waiting for message ..");
//!    let (data, mut rctx) = server.wait();
//!
//!    println!("Server received: '{}'", data);
//!
//!    // Process data from client
//!
//!    // Reply to client
//!    let reply = format!("Hello, {}!", data);
//!    println!("Server replying '{}'", reply);
//!    rctx.reply(reply);
//!
//!    println!("Server done");
//!  });
//!
//!  let msg = String::from("Client");
//!  println!("Client sending '{}'", msg);
//!  let reply = client.send(String::from(msg)).unwrap();
//!  println!("Client received reply '{}'", reply);
//!  println!("Client done");
//!
//!  server_thread.join().unwrap();
//! }
//! ```
//! In practice the send/reply types will probably be `enum`s used to
//! indicate command/return type with associated data.  The third type argument
//! to [`channel`] is an error type that can be used to explicitly pass errors
//! back to the sender.
//!
//! # Semantics
//! There are some potentially useful semantic quirks that can be good to know
//! about, but some of them should be used with caution.  This section will
//! describe some semantics that you can rely on, and others that you should be
//! careful about relying on.
//!
//! ## Stable invariants
//!
//! These are behaviors which should not change in future versions.
//!
//! - The reply contexts are independent of the `Server` context.  This has
//!   some useful implications for server threads that spawn separate threads
//!   to process messages and return replies:  *The server can safely terminate
//!   while there are clients waiting for replies* (implied: the server can
//!   safely terminate while there are reply contexts in-flight).
//! - A cloned client is paired with the same server as its origin, but in all
//!   other respects the clone and its origin are independent of each other.
//! - A client can be moved to a new thread.
//! - Any permutation of sync/async server/clients can be combined.  `async`
//!   code must use the async method variants when available.
//!
//! ## Unstable invariants
//!
//! These are invariants you can trust will work in the current version, but
//! they exist merely as a side-effect of the current implementation.  Avoid
//! relying on these if possible.
//!
//! - A single client can be used from two different threads.  If a `Client`
//!   object in placed in an Arc, is cloned and passed to another thread/task
//!   then both the clone and the original can be used simultaneously.  In the
//!   future this may not be allowed. It is recommended that a new clone of the
//!   client be created instead.

mod client;
mod err;
mod rctx;
mod server;

pub use err::Error;

use std::sync::Arc;

use sigq::Queue as NotifyQueue;

pub use crate::client::Client;
pub use crate::rctx::ReplyContext;
pub use crate::server::Server;

/// Create a pair of linked [`Server`] and [`Client`] objects.
///
/// The [`Server`] object is used to wait for incoming messages from connected
/// clients.  Once a message arrives it must reply to it using a
/// [`ReplyContext`] that's returned to it in the same call that returned the
/// message.
///
/// The [`Client`] object can be used to send messages to the [`Server`].  The
/// [`Client::send()`] call will not return until the server has replied.
///
/// Clients can be [cloned](Client::clone()); each clone will create a
/// new client object that is connected to the same server object, but is
/// completely independent of the original client.
///
/// The `S` type parameter is the "send" data type that clients will transfer
/// to the server.  The `R` type parameter is the "receive" data type that
/// clients will receive from the server.  The `E` type parameter can be used
/// to return application specific errors from the server to the client.
pub fn channel<S, R, E>() -> (Server<S, R, E>, Client<S, R, E>) {
  let srvq = Arc::new(NotifyQueue::new());
  let server = Server {
    srvq: Arc::clone(&srvq)
  };

  // Note: The client stores a weak reference to the server object
  let client = Client {
    srvq: Arc::downgrade(&srvq)
  };

  (server, client)
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
