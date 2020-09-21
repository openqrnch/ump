//! Micro Message Pass (ump) is a library that has some similarities with the
//! common mpsc channel libraries.  The most notable difference is that in ump
//! the channel is bidirectional.  ump uses the terms "client"/"server"
//! rather than "tx"/"rx", and each message pass from a client to a server
//! requires a response from the server.
//!
//! The primary purpose of ump is to create simple RPC like designs, but
//! between threads/tasks in a process rather than between processes over a
//! network.
//!
//! # Example
//! ```
//! use std::thread;
//!
//! use ump::channel;
//!
//! fn main() {
//!  let (mut server, mut client) = channel::<String, String>();
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
//!
//! In practice it's more likely that the channel types are `enum`s used to
//! indicate command/return type with associated data.

mod client;
mod err;
mod nq;
mod rctx;
mod server;
mod srvq;

pub use err::Error;

use std::sync::Arc;

use crate::client::Client;
use crate::nq::NotifyQueue;
use crate::server::Server;

/// Create a pair of linked `Server` and `Client` object.
///
/// The `Server` object is used to wait for incoming messages from connected
/// clients.  Once a message arrives it must reply to it using a reply context
/// that's returned to it in the same call that returned the message.
///
/// The `Client` object can be used to send messages to the `Server.  The
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
