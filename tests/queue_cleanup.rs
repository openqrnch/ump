// Make sure that the InnerReplyContext aborts on Drop if object is still
// queued.
use std::{thread, time};

use ump::{channel, Error};

#[test]
fn sync_expect_server_death() {
  let (server, client) = channel::<String, String>();

  let server_thread = thread::spawn(move || {
    // Should be doing something more robust ..
    let one_second = time::Duration::from_secs(1);
    thread::sleep(one_second);
    drop(server);
  });

  let msg = String::from("Client");
  let reply = client.send(String::from(msg));
  match reply {
    Err(Error::ServerDisappeared) => {
      // This is the expected error
    }
    _ => {
      panic!("Unexpected return value");
    }
  }

  server_thread.join().unwrap();
}


#[test]
fn async_expect_server_death() {
  let mut tokrt = tokio::runtime::Runtime::new().unwrap();

  let (server, client) = channel::<String, String>();

  let server_thread = thread::spawn(move || {
    // Should be doing something more robust ..
    let one_second = time::Duration::from_secs(1);
    thread::sleep(one_second);
    drop(server);
  });

  tokrt.block_on(async {
    let msg = String::from("Client");
    let reply = client.asend(msg).await;
    //let reply = client.send(msg);
    match reply {
      Err(Error::ServerDisappeared) => {
        // This is the expected error
      }
      _ => {
        panic!("Unexpected return value");
      }
    }
  });

  server_thread.join().unwrap();
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
