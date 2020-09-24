use std::env;
use std::thread;

use ump::channel;

// Run several clients, but each client iterates only once.
//
// - Get number of requested clients from command line
// - Start a server on a thread
//   - Server will increase a counter each time a message has been
//     received/processed.  Once the number of messages processed reaches the
//     number of requested clients the server thread will self-terminate.
// - Launch the requested number of client threads
// - Join all the thread handles
fn main() {
  // Get number of client threads to kick off.  Default to two.
  let args: Vec<String> = env::args().collect();
  let nclients = if args.len() > 1 {
    args[1].parse::<usize>().unwrap()
  } else {
    2
  };

  // Create server and original client
  let (server, client) = channel::<String, String>();

  // Launch server thread
  let server_thread = thread::spawn(move || {
    let mut count = 0;

    // Keep looping until each client as sent a message
    while count < nclients {
      // Wait for data to arrive from a client
      println!("Server waiting for message ..");
      let (data, rctx) = server.wait();

      println!("Server received: '{}'", data);

      // .. process data from client ..

      // Reply to client
      let reply = format!("Hello, {}!", data);
      println!("Server replying '{}'", reply);
      rctx.reply(reply).unwrap();

      // Increase message counter
      count += 1;
    }
    println!("Server done");
  });

  let mut join_handles = Vec::new();
  for i in 0..nclients {
    let client_clone = client.clone();
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
