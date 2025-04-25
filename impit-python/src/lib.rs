use pyo3::prelude::*;

mod async_client;
mod client;
mod request;
mod response;
mod errors;

use async_client::AsyncClient;
use client::Client;
use request::RequestBody;
use std::collections::HashMap;


#[pymodule]
fn impit(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_class::<AsyncClient>()?;
    m.add_class::<response::ImpitPyResponse>()?;

    m.add("HttpError", m.py().get_type::<errors::HttpError>())?;
    m.add("HttpStatusError", m.py().get_type::<errors::HttpStatusError>())?;
    m.add("RequestError", m.py().get_type::<errors::RequestError>())?;
    m.add("TransportError", m.py().get_type::<errors::TransportError>())?;
    m.add("UnsupportedProtocol", m.py().get_type::<errors::UnsupportedProtocol>())?;
    m.add("TooManyRedirects", m.py().get_type::<errors::TooManyRedirects>())?;
    m.add("InvalidUrl", m.py().get_type::<errors::InvalidUrl>())?;
    
    macro_rules! http_no_client {
        ($($name:ident),*) => {
            $(
                #[pyfunction]
                #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
                fn $name(
                    url: String,
                    content: Option<Vec<u8>>,
                    data: Option<RequestBody>,
                    headers: Option<HashMap<String, String>>,
                    timeout: Option<f64>,
                    force_http3: Option<bool>,
                ) -> Result<response::ImpitPyResponse, errors::ImpitPyError> {
                    let mut client = Client::new(None, None, None, None, None, None, None, None);

                    client.$name(url, content, data, headers, timeout, force_http3)
                }

                m.add_function(wrap_pyfunction!($name, m)?)?;
            )*
        };
    }

    http_no_client!(get, post, put, head, patch, delete, options, trace);

    Ok(())
}
