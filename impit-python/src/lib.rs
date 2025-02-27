use pyo3::prelude::*;

mod client;
mod response;
use client::Client;

use std::collections::HashMap;

macro_rules! http_no_client {
    ($($name:ident),*) => {
        $(
            #[pyfunction]
            fn $name(
                url: String,
                content: Option<Vec<u8>>,
                data: Option<HashMap<String, String>>,
                headers: Option<HashMap<String, String>>,
                timeout: Option<f64>,
            ) -> response::ImpitPyResponse {
                let mut client = Client::new(None, None, None, None, None);

                client.$name(url, content, data, headers, timeout)
            }
        )*
    };
}

http_no_client!(get, post, put, head, patch, delete, options, trace);

#[pymodule]
fn impit(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;

    macro_rules! register_http_no_client {
        ($($name:ident),*) => {
            $(
                m.add_function(wrap_pyfunction!($name, m)?)?;
            )*
        };
    }

    register_http_no_client!(get, post, put, head, patch, delete, options, trace);

    Ok(())
}
