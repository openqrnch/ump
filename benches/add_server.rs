use std::thread;

use criterion::{criterion_group, criterion_main, Criterion};

use ump::channel;

enum Ops {
  Die,
  Add(i32, i32),
  AddThreaded(i32, i32)
}

pub fn criterion_benchmark(c: &mut Criterion) {
  let (server, client) = channel();

  let server_thread = thread::spawn(move || {
    let mut croak = false;

    while croak == false {
      let (data, rctx) = server.wait();
      match data {
        Ops::Die => {
          croak = true;
          rctx.reply(0).unwrap();
        }
        Ops::Add(a, b) => rctx.reply(a + b).unwrap(),
        Ops::AddThreaded(a, b) => {
          thread::spawn(move || {
            rctx.reply(a + b).unwrap();
          });
        }
      }
    }
  });

  let mut p: i32 = 0;
  let mut q: i32 = 0;

  c.bench_function("add", |b| {
    b.iter(|| {
      p += 2;
      q -= 3;
      let result = client.send(Ops::Add(p, q)).unwrap();
      assert_eq!(result, q + p);
    })
  });

  p = 0;
  q = 0;
  c.bench_function("add (threaded)", |b| {
    b.iter(|| {
      p += 2;
      q -= 3;
      let result = client.send(Ops::AddThreaded(p, q)).unwrap();
      assert_eq!(result, q + p);
    })
  });

  let result = client.send(Ops::Die).unwrap();
  assert_eq!(result, 0);

  server_thread.join().unwrap();
}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
