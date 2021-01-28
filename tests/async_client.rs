use std::thread;

use ump::channel;

enum Request {
  Add(i32, i32),
  Croak
}

enum Reply {
  Sum(i32),
  OkICroaked
}

#[test]
fn async_client() {
  let tokrt = tokio::runtime::Runtime::new().unwrap();

  let niterations = 256;

  let (server, client) = channel::<Request, Reply, ()>();

  let server_thread = thread::spawn(move || loop {
    let (req, rctx) = server.wait();
    match req {
      Request::Add(a, b) => rctx.reply(Reply::Sum(a + b)).unwrap(),
      Request::Croak => {
        rctx.reply(Reply::OkICroaked).unwrap();
        break;
      }
    }
  });

  tokrt.block_on(async {
    let mut a: i32 = 0;
    let mut b: i32 = 0;

    for _ in 0..niterations {
      a += 2;
      b -= 3;
      let result = client.asend(Request::Add(a, b)).await.unwrap();
      if let Reply::Sum(sum) = result {
        assert_eq!(sum, a + b);
      } else {
        panic!("Didn't get sum");
      }
    }
    let result = client.asend(Request::Croak).await.unwrap();
    if let Reply::OkICroaked = result {
    } else {
      panic!("Didn't get a croak");
    }
  });

  server_thread.join().unwrap();
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
