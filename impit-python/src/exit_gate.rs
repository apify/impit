//! Keeps tokio threads out of the interpreter once it starts shutting down.
//!
//! On CPython < 3.13.8, a non-Python thread that re-takes the GIL during finalization is killed
//! with `pthread_exit`, which aborts the process when there are Rust frames on its stack. A result
//! delivery from `future_into_py` releases the GIL inside `call_soon_threadsafe` (`socket.send`),
//! so if the awaiting coroutine was the last thing the program did, the interpreter can start
//! finalizing before the delivering thread gets the GIL back. `atexit` hooks run before the
//! interpreter is marked as finalizing, so we close the gate there and wait for in-flight
//! deliveries to leave Python.

use std::cell::OnceCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Condvar, Mutex};
use std::time::Duration;

use pyo3::prelude::*;
use pyo3_async_runtimes::generic::{ContextExt, Runtime};
use pyo3_async_runtimes::TaskLocals;

const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

struct State {
    closed: bool,
    in_flight: usize,
}

static STATE: Mutex<State> = Mutex::new(State {
    closed: false,
    in_flight: 0,
});
static DRAINED: Condvar = Condvar::new();

struct InFlight;

impl InFlight {
    fn enter() -> Option<Self> {
        let mut state = STATE.lock().unwrap();
        if state.closed {
            return None;
        }
        state.in_flight += 1;
        Some(InFlight)
    }
}

impl Drop for InFlight {
    fn drop(&mut self) {
        let mut state = STATE.lock().unwrap();
        state.in_flight -= 1;
        if state.in_flight == 0 {
            DRAINED.notify_all();
        }
    }
}

#[pyfunction]
fn close(py: Python<'_>) {
    py.detach(|| {
        let mut state = STATE.lock().unwrap();
        state.closed = true;
        let _ = DRAINED.wait_timeout_while(state, DRAIN_TIMEOUT, |s| s.in_flight > 0);
    });
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Newer versions hang such threads instead of killing them.
    if m.py().version_info() >= (3, 13, 8) {
        return Ok(());
    }
    let close = wrap_pyfunction!(close, m)?;
    m.py()
        .import("atexit")?
        .call_method1("register", (close,))?;
    Ok(())
}

struct GatedTokio;

tokio::task_local! {
    static TASK_LOCALS: OnceCell<TaskLocals>;
}

impl Runtime for GatedTokio {
    type JoinError = tokio::task::JoinError;
    type JoinHandle = tokio::task::JoinHandle<()>;

    fn spawn<F>(fut: F) -> Self::JoinHandle
    where
        F: Future<Output = ()> + Send + 'static,
    {
        pyo3_async_runtimes::tokio::get_runtime().spawn(fut)
    }

    // pyo3-async-runtimes delivers every result (and panic) to Python from a closure passed here.
    // Once the gate is closed, the closure is dropped without attaching, and pyo3 defers the
    // decrefs of the Python objects it holds.
    fn spawn_blocking<F>(f: F) -> Self::JoinHandle
    where
        F: FnOnce() + Send + 'static,
    {
        pyo3_async_runtimes::tokio::get_runtime().spawn_blocking(move || {
            if let Some(_in_flight) = InFlight::enter() {
                f();
            }
        })
    }
}

impl ContextExt for GatedTokio {
    fn scope<F, R>(locals: TaskLocals, fut: F) -> Pin<Box<dyn Future<Output = R> + Send>>
    where
        F: Future<Output = R> + Send + 'static,
    {
        let cell = OnceCell::new();
        cell.set(locals).unwrap();
        Box::pin(TASK_LOCALS.scope(cell, fut))
    }

    fn get_task_locals() -> Option<TaskLocals> {
        TASK_LOCALS
            .try_with(|c| c.get().cloned())
            .unwrap_or_default()
    }
}

/// Drop-in replacement for `pyo3_async_runtimes::tokio::future_into_py` that goes through the
/// exit gate.
pub fn future_into_py<F, T>(py: Python<'_>, fut: F) -> PyResult<Bound<'_, PyAny>>
where
    F: Future<Output = PyResult<T>> + Send + 'static,
    T: for<'py> IntoPyObject<'py> + Send + 'static,
{
    // Python code is still running, so the interpreter isn't finalizing yet. This reopens the gate
    // for atexit hooks that run after ours, whose results would never arrive otherwise.
    STATE.lock().unwrap().closed = false;
    pyo3_async_runtimes::generic::future_into_py::<GatedTokio, _, T>(py, fut)
}
