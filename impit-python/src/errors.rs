use std::error::Error;

use pyo3::create_exception;

create_exception!(impit, HttpError, pyo3::exceptions::PyException);
create_exception!(impit, HttpStatusError, HttpError);
create_exception!(impit, RequestError, HttpError);
create_exception!(impit, TransportError, RequestError);
create_exception!(impit, UnsupportedProtocol, TransportError);
create_exception!(impit, TooManyRedirects, RequestError);
create_exception!(impit, InvalidUrl, pyo3::exceptions::PyException);

pub(crate) enum ImpitPyError {
    HttpError,
    HttpStatusError(String),
    RequestError(String),
    TransportError(String),
    UnsupportedProtocol(String),
    TooManyRedirects(String),
    InvalidUrl(String),
}

impl From<impit::impit::ErrorType> for ImpitPyError {
    fn from(err: impit::impit::ErrorType) -> Self {
        match err {
            impit::impit::ErrorType::RequestError(err) => {
                match err.source() {
                    Some(source) => {
                        if format!("{:?}", source).contains("TooManyRedirects") {
                            ImpitPyError::TooManyRedirects(format!("{:?}", source))
                        } else {
                            ImpitPyError::RequestError(format!("{:?}", source))
                        }
                    }
                    None => ImpitPyError::RequestError(format!("{:?}", err)),
                }
            },
            impit::impit::ErrorType::Http3Disabled => ImpitPyError::RequestError("HTTP3 is disabled".to_string()),
            impit::impit::ErrorType::InvalidMethod(err) => ImpitPyError::RequestError(format!("{}", err)),
            impit::impit::ErrorType::UrlProtocolError(_) => ImpitPyError::UnsupportedProtocol(format!("{}", err)),
            _ => ImpitPyError::InvalidUrl(format!("{}", err)),
        }
    }
}

impl Into<pyo3::PyErr> for ImpitPyError {
    fn into(self) -> pyo3::PyErr {
        match self {
            ImpitPyError::HttpError => HttpError::new_err("HTTP error"),
            ImpitPyError::HttpStatusError(msg) => HttpStatusError::new_err(msg),
            ImpitPyError::RequestError(msg) => RequestError::new_err(msg),
            ImpitPyError::TooManyRedirects(msg) => TooManyRedirects::new_err(msg),
            ImpitPyError::InvalidUrl(msg) => InvalidUrl::new_err(msg),
            ImpitPyError::TransportError(msg) => TransportError::new_err(msg),
            ImpitPyError::UnsupportedProtocol(msg) => UnsupportedProtocol::new_err(msg),
        }
    }
}
