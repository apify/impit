use ::impit::impit::{ErrorType, ImpitBuilder};
use std::{collections::HashMap, time::Duration};

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
    Get,
    Options,
    Head,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Default)]
struct RequestKwargs {
    _params: Option<Vec<(String, String)>>,
    _data: Option<Vec<u8>>,
    _headers: Option<Vec<(String, String)>>,
    _cookies: Option<HashMap<String, String>>,
    _timeout: Option<Duration>,
    _allow_redirects: Option<bool>,
    _proxies: Option<HashMap<String, String>>,
    _verify: Option<bool>,
}

async fn request(
    method: HttpMethods,
    url: String,
    _kwargs: Option<RequestKwargs>,
) -> Result<ImpitResponse, ErrorType> {
    let mut impit = ImpitBuilder::default().build();

    let response = match method {
        HttpMethods::Get => impit.get(url, None).await,
        HttpMethods::Options => impit.options(url, None).await,
        HttpMethods::Head => impit.head(url, None).await,
        HttpMethods::Post => impit.post(url, None, None).await,
        HttpMethods::Put => impit.put(url, None, None).await,
        HttpMethods::Patch => impit.patch(url, None, None).await,
        HttpMethods::Delete => impit.delete(url, None).await,
    }?;

    let status_code = response.status().as_u16();
    let text = response.text().await.unwrap();

    Ok(ImpitResponse { status_code, text })
}

fn request_sync(
    method: HttpMethods,
    url: String,
    kwargs: Option<RequestKwargs>,
) -> Result<ImpitResponse, pyo3::PyErr> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        request(method, url, kwargs)
            .await
            .map_err(|e| PyErr::new::<PyAny, String>(format!("{:?}", e)))
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
        get => Get,
        head => Head,
        post => Post,
        patch => Patch,
        put => Put,
        delete => Delete,
        options => Options
    }

    Ok(())
}
