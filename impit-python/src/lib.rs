use pyo3::prelude::*;

mod client;
mod async_client;
mod response;
use client::Client;
use async_client::AsyncClient;

use std::collections::HashMap;

#[pymodule]
fn impit(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_class::<AsyncClient>()?;

    macro_rules! http_no_client {
        ($($name:ident),*) => {
            $(
                #[pyfunction]
                #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
                fn $name(
                    url: String,
                    content: Option<Vec<u8>>,
                    data: Option<HashMap<String, String>>,
                    headers: Option<HashMap<String, String>>,
                    timeout: Option<f64>,
                    force_http3: Option<bool>,
                ) -> response::ImpitPyResponse {
                    let mut client = Client::new(None, None, None, None, None);

                    client.$name(url, content, data, headers, timeout, force_http3)
                }

                m.add_function(wrap_pyfunction!($name, m)?)?;
            )*
        };
    }

    http_no_client!(get, post, put, head, patch, delete, options, trace);

    Ok(())
}
