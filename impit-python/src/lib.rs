use pyo3::prelude::*;

mod client;
mod response;
use client::Client;

#[pymodule]
fn impit(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;

    Ok(())
}
