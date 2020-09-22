use std::thread;

use ump::channel;

fn main() {
  let (server, client) = channel::<String, String>();

  let server_thread = thread::spawn(move || {
    // Wait for data to arrive from a client
    println!("Server waiting for message ..");
    let (data, cctx) = server.wait();

    println!("Server received: '{}'", data);

    // Process data from client

    // Reply to client
    let reply = format!("Hello, {}!", data);
    println!("Server replying '{}'", reply);
    cctx.reply(reply).unwrap();

    println!("Server done");
  });

  let msg = String::from("Client");
  println!("Client sending '{}'", msg);
  let reply = client.send(String::from(msg)).unwrap();
  println!("Client received reply '{}'", reply);
  println!("Client done");

  server_thread.join().unwrap();
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
