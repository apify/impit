use napi::bindgen_prelude::Result;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

/// A Tokio stream wrapper that can be aborted via a channel
pub(crate) struct AbortableStream<S> {
  inner: Pin<Box<S>>,
  abort_rx: Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<()>>>>,
}

impl<S> AbortableStream<S> {
  pub(crate) fn new(
    inner: S,
    abort_rx: Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<()>>>>,
  ) -> Self {
    Self {
      inner: Box::pin(inner),
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
    if let Ok(mut guard) = self.abort_rx.try_lock() {
      if let Some(ref mut rx) = *guard {
        match rx.poll_recv(cx) {
          Poll::Ready(Some(_)) => {
            return Poll::Ready(Some(Err(napi::Error::new(
              napi::Status::GenericFailure,
              "impit: Request was aborted through AbortSignal",
            ))));
          }
          _other => {}
        }
      }
    }

    self.inner.as_mut().poll_next(cx)
  }
}
