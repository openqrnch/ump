use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll};
use std::thread;

use crate::rctx::err::Error;

pub(crate) enum State<I> {
  Waiting,
  Item(I),
  Finalized,
  Aborted
}

pub struct InnerReplyContext<I> {
  pub(crate) signal: Arc<Condvar>,
  pub(crate) data: Arc<Mutex<State<I>>>
}

impl<I: 'static + Send> InnerReplyContext<I> {
  pub fn new() -> Self {
    InnerReplyContext {
      signal: Arc::new(Condvar::new()),
      data: Arc::new(Mutex::new(State::Waiting))
    }
  }

  pub fn put(&self, item: I) {
    let mut mg = self.data.lock().unwrap();
    *mg = State::Item(item);
    drop(mg);

    self.signal.notify_one();
  }

  pub fn get(&self) -> Result<I, Error> {
    let mut mg = self.data.lock().unwrap();

    let msg = loop {
      match &*mg {
        State::Waiting => {
          // Still waiting for server to report back with data
          mg = self.signal.wait(mg).unwrap();
          continue;
        }
        State::Item(_msg) => {
          // Set Finalized state and return item
          if let State::Item(msg) =
            std::mem::replace(&mut *mg, State::Finalized)
          {
            break msg;
          } else {
            // We're *really* in trouble if this happens ..
            panic!("Unexpected state; not State::Message()");
          }
        }
        State::Finalized => {
          // We're *really* in trouble if this happens ..
          panic!("Unexpected state State::Finalized");
        }
        State::Aborted => {
          return Err(Error::Aborted);
        }
      }
    };
    drop(mg);

    Ok(msg)
  }

  pub fn aget(&self) -> WaitReplyFuture<I> {
    WaitReplyFuture::new(self)
  }
}

impl<T> Clone for InnerReplyContext<T> {
  fn clone(&self) -> Self {
    InnerReplyContext {
      signal: Arc::clone(&self.signal),
      data: Arc::clone(&self.data)
    }
  }
}


pub struct WaitReplyFuture<I> {
  signal: Arc<Condvar>,
  data: Arc<Mutex<State<I>>>
}

impl<I> WaitReplyFuture<I> {
  fn new(irctx: &InnerReplyContext<I>) -> Self {
    WaitReplyFuture {
      signal: Arc::clone(&irctx.signal),
      data: Arc::clone(&irctx.data)
    }
  }
}

impl<I: 'static + Send> Future for WaitReplyFuture<I> {
  type Output = Result<I, Error>;
  fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
    let mut state = self.data.lock().unwrap();
    match &*state {
      State::Waiting => {
        let waker = ctx.waker().clone();
        let data = Arc::clone(&self.data);
        let signal = Arc::clone(&self.signal);
        thread::spawn(move || {
          let mut istate = data.lock().unwrap();
          if let State::Waiting = *istate {
            istate = signal.wait(istate).unwrap();
          }
          drop(istate);
          waker.wake();
        });
        drop(state);
        Poll::Pending
      }
      State::Item(_msg) => {
        if let State::Item(msg) =
          std::mem::replace(&mut *state, State::Finalized)
        {
          Poll::Ready(Ok(msg))
        } else {
          // We're *really* in trouble if this happens ..
          panic!("Unexpected state; not State::Message()");
        }
      }
      State::Finalized => {
        panic!("Unexpected state");
      }
      State::Aborted => Poll::Ready(Err(Error::Aborted))
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
