use napi::threadsafe_function::ThreadsafeFunction;
use tokio::sync::oneshot;

use std::thread;
use std::time::Duration;

use std::thread::sleep;

use napi::Status;

use napi::bindgen_prelude::Promise;

use napi::bindgen_prelude::FromNapiValue;

use napi::bindgen_prelude::JsValuesTupleIntoVec;

pub(crate) fn await_promise<
  T: Send,
  CallbackArgs: JsValuesTupleIntoVec,
  RustReturn: FromNapiValue + std::fmt::Debug + Sync + Send,
>(
  tsfn: &ThreadsafeFunction<T, Promise<RustReturn>, CallbackArgs, Status, false>,
  args: T,
) -> Result<RustReturn, napi::Error> {
  thread::scope(|scope| {
    let (tx, mut rx) = oneshot::channel();

    scope.spawn(move || match tokio::runtime::Runtime::new() {
      Ok(runtime) => {
        runtime.block_on(async {
          match tsfn.call_async(args).await {
            Ok(result) => {
              let _ = tx.send(result.await);
            }
            Err(e) => {
              let _ = tx.send(Err(napi::Error::new(
                napi::Status::GenericFailure,
                format!("[impit] failed to retrieve cookies from the external cookie store: {e}"),
              )));
            }
          }
        });
      }
      Err(e) => {
        let _ = tx.send(Err(napi::Error::new(
          napi::Status::GenericFailure,
          format!("[impit] failed to retrieve cookies from the external cookie store: {e}"),
        )));
      }
    });

    let mut result = rx.try_recv();

    let max_retries = 5;
    let mut retries = 0;

    while result.is_err() && retries < max_retries {
      sleep(Duration::from_millis(5));
      result = rx.try_recv();
      retries += 1;
    }

    match result {
      Ok(Ok(result)) => Ok(result),
      Ok(Err(e)) => Err(e),
      Err(_) => Err(napi::Error::new(
        napi::Status::GenericFailure,
        "[impit] failed to retrieve cookies from the external cookie store".to_string(),
      )),
    }
  })
}
