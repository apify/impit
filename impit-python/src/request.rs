use std::collections::HashMap;

use pyo3::{Bound, FromPyObject, PyAny};

#[derive(FromPyObject)]
pub(crate) enum RequestBody<'py> {
    #[pyo3(transparent, annotation = "bytes")]
    Bytes(Vec<u8>),
    #[pyo3(transparent, annotation = "dict[str, str]")]
    Form(HashMap<String, String>),
    #[pyo3(transparent)]
    CatchAll(Bound<'py, PyAny>), // This extraction never fails
}

pub fn form_to_bytes(data: HashMap<String, String>) -> Vec<u8> {
    let mut body = Vec::new();
    for (key, value) in data {
        body.extend_from_slice(urlencoding::encode(key.as_str()).as_bytes());
        body.extend_from_slice("=".as_bytes());
        body.extend_from_slice(urlencoding::encode(value.as_str()).as_bytes());
        body.extend_from_slice("&".as_bytes());
    }
    body.pop(); // Remove the last "&"
    body
}
