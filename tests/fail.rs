// Make sure that the ReplyContext aborts on Drop of no reply was sent.
use std::thread;

use ump::{channel, Error};

#[derive(Debug, PartialEq)]
enum MyError {
  SomeError(String)
}

#[test]
fn sync_expect_noreply() {
  let (server, client) = channel::<String, String, MyError>();

  let server_thread = thread::spawn(move || {
    // Wait for data to arrive from a client
    let (_, rctx) = server.wait();

    rctx.fail(MyError::SomeError("failed".to_string())).unwrap();
  });

  let msg = String::from("Client");
  let reply = client.send(String::from(msg));
  match reply {
    Err(Error::App(MyError::SomeError(s))) => {
      assert_eq!(s, "failed");
    }
    _ => {
      panic!("Unexpected return value");
    }
  }

  server_thread.join().unwrap();
}


#[test]
fn async_expect_noreply() {
  let tokrt = tokio::runtime::Runtime::new().unwrap();

  let (server, client) = channel::<String, String, MyError>();

  let server_thread = thread::spawn(move || {
    // Wait for data to arrive from a client
    let (_, rctx) = server.wait();

    rctx.fail(MyError::SomeError("failed".to_string())).unwrap();
  });

  tokrt.block_on(async {
    let msg = String::from("Client");
    let reply = client.asend(msg).await;
    match reply {
      Err(Error::App(MyError::SomeError(s))) => {
        assert_eq!(s, "failed");
      }
      _ => {
        panic!("Unexpected return value");
      }
    }
  });

  server_thread.join().unwrap();
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
