use std::thread;

use ump::{channel, Error};

#[test]
fn expect_abort() {
  let (server, client) = channel::<String, String>();

  let server_thread = thread::spawn(move || {
    // Wait for data to arrive from a client
    let (_, cctx) = server.wait();

    // Don't do this.
    drop(cctx);
  });

  let msg = String::from("Client");
  let reply = client.send(String::from(msg));
  match reply {
    Err(Error::Aborted) => {
      // This is the expected error
    }
    _ => {
      panic!("Unexpected return value");
    }
  }

  server_thread.join().unwrap();
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
