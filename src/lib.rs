//! Micro Message Pass (ump) is a library for passing messages between
//! thread/tasks.  It has some similarities with the common mpsc channel
//! libraries, but with the most notable difference that in `ump` the channel
//! is bidirectional.  The terms "client"/"server" are used rather than
//! "tx"/"rx".  In `ump` the client initiates all message transfers, and every
//! message pass from a client to a server requires a response from the server.
//!
//! The primary purpose of ump is to create simple RPC like designs, but
//! between threads/tasks within a process rather than between processes over
//! networks.
//!
//! # High-level usage overview
//! An application calls [`channel`](fn.channel.html) to create a linked pair
//! of a [`Server`](struct.Server.html) and a [`Client`](struct.Client.html).
//!
//! The server calls [`Server::wait()`](struct.Server.html#method.wait), which
//! blocks and waits for an incoming message from a client.
//!
//! A client, on a separate thread, calls
//! [`Client::send()`](struct.Client.html#method.send) to send a message to the
//! server.
//!
//! The server's call to `wait()` returns two objects:  The message sent by the
//! client, and a [`ReplyContext`](struct.ReplyContext.html).  After processing
//! its application-defined message, the server *must* call the
//! [`ReplyContext::reply()`](struct.ReplyContext.html#method.reply) on the
//! returned reply context object to return a reply message to the client.
//! Typically the server calls `wait()` again to wait for next message from a
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
//!  let (server, client) = channel::<String, String>();
//!
//!  let server_thread = thread::spawn(move || {
//!    // Wait for data to arrive from a client
//!    println!("Server waiting for message ..");
//!    let (data, mut cctx) = server.wait();
//!
//!    println!("Server received: '{}'", data);
//!
//!    // Process data from client
//!
//!    // Reply to client
//!    let reply = format!("Hello, {}!", data);
//!    println!("Server replying '{}'", reply);
//!    cctx.reply(reply);
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
//! (In practice it's more likely that the channel types are `enum`s used to
//! indicate command/return type with associated data).
//!
//! # Sharing Clients
//! Clone `Client` object to use multiple clients against a single `Server`.
//! While it is _currently_ possible to share a single `Client` among multiple
//! threads, *this may not be allowed in the future*.  I.e. future versions may
//! not allow the pattern `Arc<Client<S, R>>` to share a client among multiple
//! threads.  Instead, clone and pass the ownership of clones to other threads.
//!
//! # Semantics
//! The reply contexts are independent of the `Server` context.  This has some
//! useful implications for server threads that spawn separate threads to
//! process messages and return replies:  *The server can safely terminate
//! while there are clients waiting for replies* (implied: the server can
//! safely terminate while there are reply contexts in-flight).

mod client;
mod err;
mod nq;
mod rctx;
mod server;
mod srvq;

pub use err::Error;

use std::sync::Arc;

pub use crate::client::Client;
use crate::nq::NotifyQueue;
pub use crate::server::Server;
pub use rctx::ReplyContext;


/// Create a pair of linked `Server` and `Client` object.
///
/// The `Server` object is used to wait for incoming messages from connected
/// clients.  Once a message arrives it must reply to it using a
/// [`ReplyContext`](struct.ReplyContext.html) that's returned to it in the
/// same call that returned the message.
///
/// The `Client` object can be used to send messages to the `Server`.  The
/// `send()` call will not return until the server has replied.
///
/// Clients can be cloned; each clone will create a new client object that is
/// connected to the same server, but is completely independent of the original
/// client.
pub fn channel<S, R>() -> (Server<S, R>, Client<S, R>) {
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
