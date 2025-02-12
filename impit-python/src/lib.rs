use std::{collections::HashMap, time::Duration};
use ::impit::impit::{ErrorType, ImpitBuilder};

use pyo3::prelude::*;


#[pyclass]
#[derive(Debug, Clone)]
struct ImpitResponse {
    #[pyo3(get)]
    status_code: u16,
    #[pyo3(get)]
    text: String,
}

enum HttpMethods {
    GET,
    OPTIONS,
    HEAD,
    POST,
    PUT,
    PATCH,
    DELETE
}

#[derive(Default)]
struct RequestKwargs {
    params: Option<Vec<(String, String)>>,
    data: Option<Vec<u8>>,
    headers: Option<Vec<(String, String)>>,
    cookies: Option<HashMap<String, String>>,
    timeout: Option<Duration>,
    allow_redirects: Option<bool>,
    proxies: Option<HashMap<String, String>>,
    verify: Option<bool>,
}

async fn request(method: HttpMethods, url: String, kwargs: Option<RequestKwargs>) -> Result<ImpitResponse, ErrorType> {
    let mut impit = ImpitBuilder::default().build();

    let response = match method {
        HttpMethods::GET => impit.get(url, None).await,
        HttpMethods::OPTIONS => impit.options(url, None).await,
        HttpMethods::HEAD => impit.head(url, None).await,
        HttpMethods::POST => impit.post(url, None, None).await,
        HttpMethods::PUT => impit.put(url, None, None).await,
        HttpMethods::PATCH => impit.patch(url, None, None).await,
        HttpMethods::DELETE => impit.delete(url, None).await,
    }?;

    let status_code = response.status().as_u16();
    let text = response.text().await.unwrap();

    Ok(ImpitResponse { status_code, text })
}

fn request_sync(method: HttpMethods, url: String, kwargs: Option<RequestKwargs>) -> Result<ImpitResponse, pyo3::PyErr> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        request(method, url, kwargs).await.map_err(|e| PyErr::new::<PyAny, String>(format!("{:?}", e)))
    })
}

#[pymodule]
fn impit(_py: Python, m: &PyModule) -> PyResult<()> {
    macro_rules! generate_pyfn {
        ($($name:ident => $method:ident),*) => {
            $(
                #[pyfunction]
                fn $name(url: String) -> PyResult<ImpitResponse> {
                    request_sync(HttpMethods::$method, url, None)
                }

                m.add_function(wrap_pyfunction!($name, m)?).unwrap();
            )*
        };
    }

    generate_pyfn! {
        get => GET,
        head => HEAD,
        post => POST,
        patch => PATCH,
        put => PUT,
        delete => DELETE,
        options => OPTIONS
    }

    Ok(())
}
