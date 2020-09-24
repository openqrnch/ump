use crate::rctx::err::Error;
use crate::rctx::inner::State;
use crate::rctx::InnerReplyContext;

/// Public-facing sender part of the `ReplyContext` object.
///
/// This is safe to pass to applications which are meant to only be able to put
/// a value through the `ReplyContext` channel, but not extract the value from
/// it.
pub struct ReplyContext<I> {
  inner: InnerReplyContext<I>,
  did_handover: bool
}

impl<I: 'static + Send> ReplyContext<I> {
  pub fn reply(mut self, data: I) -> Result<(), Error> {
    self.inner.put(data);

    self.did_handover = true;

    Ok(())
  }
}

impl<I> Drop for ReplyContext<I> {
  fn drop(&mut self) {
    if self.did_handover == false {
      let mut do_signal: bool = false;
      let mut mg = self.inner.data.lock().unwrap();
      match *mg {
        State::Waiting => {
          *mg = State::Aborted;
          do_signal = true;
        }
        _ => {}
      }
      drop(mg);
      if do_signal {
        self.inner.signal.notify_one();
      }
    }
  }
}

impl<I> From<InnerReplyContext<I>> for ReplyContext<I> {
  fn from(inner: InnerReplyContext<I>) -> Self {
    ReplyContext {
      inner: inner.clone(),
      did_handover: false
    }
  }
}

impl<I> From<&InnerReplyContext<I>> for ReplyContext<I> {
  fn from(inner: &InnerReplyContext<I>) -> Self {
    ReplyContext {
      inner: inner.clone(),
      did_handover: false
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
