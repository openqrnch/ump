// This is a little weird example.  The client can call the server to get
// clones of the client, which is stupid because the client used to make the
// clone call can just be cloned itself.  But this is used as an illustration
// to demonstrate that a server can return client clones of clients belonging
// to _different_ servers, which is less stupid.

use std::thread;

use ump::channel;

enum Request {
  CloneClient,
  Add(i32, i32),
  Croak
}

enum Reply {
  ClientClone(ump::Client<Request, Reply, ()>),
  Sum(i32),
  OkICroaked
}

fn main() {
  let (server, client) = channel::<Request, Reply, ()>();

  let client_blueprint = client.clone();
  let server_thread = thread::spawn(move || loop {
    let (req, rctx) = server.wait();
    match req {
      Request::CloneClient => rctx
        .reply(Reply::ClientClone(client_blueprint.clone()))
        .unwrap(),
      Request::Add(a, b) => rctx.reply(Reply::Sum(a + b)).unwrap(),
      Request::Croak => {
        rctx.reply(Reply::OkICroaked).unwrap();
        break;
      }
    }
  });

  if let Reply::ClientClone(cloned_client) =
    client.send(Request::CloneClient).unwrap()
  {
    if let Reply::Sum(x) = cloned_client.send(Request::Add(5, 7)).unwrap() {
      assert_eq!(x, 12);
    } else {
      panic!("Unexpected result");
    }
  } else {
    panic!("Unexpected result");
  }

  let _ = client.send(Request::Croak);

  server_thread.join().unwrap();
}


// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
