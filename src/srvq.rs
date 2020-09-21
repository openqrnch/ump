use crate::rctx::InnerReplyContext;

pub(crate) struct ServerQueueNode<S, R> {
  /// Raw message being sent from the client to the server.
  pub(crate) msg: S,

  /// Keep track of data needed to share reply data.
  pub(crate) reply: InnerReplyContext<R>
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
