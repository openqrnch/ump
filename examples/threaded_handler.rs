use std::env;
use std::thread;

use ump::channel;

// This is basically the same test as many_once, but the server launches a new
// thread to process and reply to client requests.
fn main() {
  // Get number of client threads to kick off.  Default to two.
  let args: Vec<String> = env::args().collect();
  let nclients = if args.len() > 1 {
    args[1].parse::<usize>().unwrap()
  } else {
    2
  };

  // Create server and original client
  let (mut server, client) = channel::<String, String>();

  // Launch server thread
  let server_thread = thread::spawn(move || {
    let mut count = 0;

    // Keep looping until each client as sent a message
    while count < nclients {
      // Wait for data to arrive from a client
      println!("Server waiting for message ..");
      let (data, cctx) = server.wait();

      // Move the received data and reply context into a thread to allow other
      // messages to be received while processing this message.
      thread::spawn(move || {
        println!("Server received: '{}'", data);

        // Process data from client

        // Reply to client
        let reply = format!("Hello, {}!", data);
        println!("Server replying '{}'", reply);
        cctx.reply(reply).unwrap();
      });

      // Increase message counter
      count += 1;
    }
    println!("Server done");
  });

  let mut join_handles = Vec::new();
  for i in 0..nclients {
    let mut client_clone = client.clone();
    let client_thread = thread::spawn(move || {
      let name = format!("Client {}", i + 1);
      let msg = String::from(&name);
      println!("{} sending '{}'", name, msg);
      let reply = client_clone.send(String::from(msg)).unwrap();
      println!("{} received reply '{}' -- done", name, reply);
    });
    join_handles.push(client_thread);
  }

  for n in join_handles {
    n.join().unwrap();
  }
  server_thread.join().unwrap();
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
