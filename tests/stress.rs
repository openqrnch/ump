use std::thread;

use ump::channel;

enum Ops {
  Die,
  Add(i32, i32),
  Sub(i32, i32)
}

#[test]
fn one_at_a_time() {
  let (server, client) = channel::<Ops, i32, ()>();

  let server_thread = thread::spawn(move || {
    let mut croak = false;

    while croak == false {
      let (data, rctx) = server.wait();
      match data {
        Ops::Die => {
          croak = true;
          rctx.reply(0).unwrap();
        }
        Ops::Add(a, b) => {
          rctx.reply(a + b).unwrap();
        }
        Ops::Sub(a, b) => {
          rctx.reply(a - b).unwrap();
        }
      }
    }
  });

  let mut a: i32 = 0;
  let mut b: i32 = 0;

  for _ in 0..65535 {
    a += 2;
    b -= 3;
    let result = client.send(Ops::Add(a, b)).unwrap();
    assert_eq!(result, a + b);
  }
  let result = client.send(Ops::Die).unwrap();
  assert_eq!(result, 0);

  server_thread.join().unwrap();
}

#[test]
fn one_at_a_time_threaded_handler() {
  let (server, client) = channel::<Ops, i32, ()>();

  let niterations = 256;

  let server_thread = thread::spawn(move || {
    let mut count = 0;
    let mut handles = Vec::new();
    // +1 because we want to wait for the croak message as well
    while count < niterations + 1 {
      let (data, rctx) = server.wait();
      let h = thread::spawn(move || match data {
        Ops::Die => {
          rctx.reply(0).unwrap();
        }
        Ops::Add(a, b) => {
          rctx.reply(a + b).unwrap();
        }
        Ops::Sub(a, b) => {
          rctx.reply(a - b).unwrap();
        }
      });
      handles.push(h);
      count += 1;
    }
    for h in handles {
      h.join().unwrap();
    }
  });

  let mut a: i32 = 0;
  let mut b: i32 = 0;

  for _ in 0..niterations {
    a += 2;
    b -= 3;
    let result = client.send(Ops::Sub(a, b)).unwrap();
    assert_eq!(result, a - b);
  }
  let result = client.send(Ops::Die).unwrap();
  assert_eq!(result, 0);

  server_thread.join().unwrap();
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
