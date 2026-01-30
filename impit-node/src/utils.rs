use napi::Status;

use napi::bindgen_prelude::{FromNapiValue, JsValuesTupleIntoVec, Promise};
use napi::threadsafe_function::ThreadsafeFunction;

pub(crate) fn await_promise<
  T: Send,
  CallbackArgs: JsValuesTupleIntoVec,
  RustReturn: FromNapiValue + std::fmt::Debug + Sync + Send,
>(
  tsfn: &ThreadsafeFunction<T, Promise<RustReturn>, CallbackArgs, Status, false>,
  args: T,
) -> Result<RustReturn, napi::Error> {
  futures::executor::block_on(async {
    match tsfn.call_async(args).await {
      Ok(result) => result.await,
      Err(e) => Err(napi::Error::new(
        napi::Status::GenericFailure,
        format!("[impit] failed to retrieve cookies from the external cookie store: {e}"),
      )),
    }
  })
}
