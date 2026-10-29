use napi::bindgen_prelude::Result;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

/// A Tokio stream wrapper that can be aborted via a channel
pub(crate) struct AbortableStream<S> {
  inner: Option<Pin<Box<S>>>,
  abort_rx: Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<()>>>>,
}

impl<S> AbortableStream<S> {
  pub(crate) fn new(
    inner: S,
    abort_rx: Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<()>>>>,
  ) -> Self {
    Self {
      inner: Some(Box::pin(inner)),
      abort_rx,
    }
  }
}

impl<S> tokio_stream::Stream for AbortableStream<S>
where
  S: tokio_stream::Stream<Item = Result<Vec<u8>>>,
{
  type Item = Result<Vec<u8>>;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let this = &mut *self;
    if let Ok(mut guard) = this.abort_rx.try_lock() {
      if let Some(ref mut rx) = *guard {
        match rx.poll_recv(cx) {
          Poll::Ready(Some(_)) => {
            // napi only drops the stream on close or cancel, not on error, so drop the body here
            // to close the connection.
            this.inner = None;
            return Poll::Ready(Some(Err(napi::Error::new(
              napi::Status::GenericFailure,
              "impit: Request was aborted through AbortSignal",
            ))));
          }
          _other => {}
        }
      }
    }

    match this.inner.as_mut() {
      Some(inner) => inner.as_mut().poll_next(cx),
      None => Poll::Ready(None),
    }
  }
}
