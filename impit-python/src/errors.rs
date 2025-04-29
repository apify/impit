use impit::errors::ImpitError;

use pyo3::create_exception;

create_exception!(impit, HTTPError, pyo3::exceptions::PyException);
create_exception!(impit, RequestError, HTTPError);
create_exception!(impit, TransportError, RequestError);
create_exception!(impit, TimeoutException, TransportError);
create_exception!(impit, ConnectTimeout, TimeoutException);
create_exception!(impit, ReadTimeout, TimeoutException);
create_exception!(impit, WriteTimeout, TimeoutException);
create_exception!(impit, PoolTimeout, TimeoutException);
create_exception!(impit, NetworkError, TransportError);
create_exception!(impit, ConnectError, NetworkError);
create_exception!(impit, ReadError, NetworkError);
create_exception!(impit, WriteError, NetworkError);
create_exception!(impit, CloseError, NetworkError);
create_exception!(impit, ProtocolError, TransportError);
create_exception!(impit, LocalProtocolError, ProtocolError);
create_exception!(impit, RemoteProtocolError, ProtocolError);
create_exception!(impit, ProxyError, TransportError);
create_exception!(impit, UnsupportedProtocol, TransportError);
create_exception!(impit, DecodingError, RequestError);
create_exception!(impit, TooManyRedirects, RequestError);
create_exception!(impit, HTTPStatusError, HTTPError);
create_exception!(impit, InvalidURL, pyo3::exceptions::PyException);
create_exception!(impit, CookieConflict, pyo3::exceptions::PyException);
create_exception!(impit, StreamError, pyo3::exceptions::PyException);
create_exception!(impit, StreamConsumed, StreamError);
create_exception!(impit, ResponseNotRead, StreamError);
create_exception!(impit, RequestNotRead, StreamError);
create_exception!(impit, StreamClosed, StreamError);

pub(crate) struct ImpitPyError(pub ImpitError);

impl From<ImpitError> for ImpitPyError {
    fn from(err: ImpitError) -> Self {
        ImpitPyError(err)
    }
}

impl From<ImpitPyError> for pyo3::PyErr {
    fn from(err: ImpitPyError) -> pyo3::PyErr {
        match err {
            ImpitPyError(ImpitError::RequestError) => RequestError::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::TransportError) => {
                TransportError::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::TimeoutException(_)) => {
                TimeoutException::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::ConnectTimeout) => {
                ConnectTimeout::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::ReadTimeout) => ReadTimeout::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::WriteTimeout) => WriteTimeout::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::PoolTimeout) => PoolTimeout::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::NetworkError) => NetworkError::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::ConnectError) => ConnectError::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::ReadError) => ReadError::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::WriteError) => WriteError::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::CloseError) => CloseError::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::ProtocolError) => ProtocolError::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::LocalProtocolError) => {
                LocalProtocolError::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::RemoteProtocolError) => {
                RemoteProtocolError::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::ProxyError(_)) => ProxyError::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::UnsupportedProtocol) => {
                UnsupportedProtocol::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::DecodingError) => DecodingError::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::TooManyRedirects(_)) => {
                TooManyRedirects::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::HTTPStatusError(_)) => {
                HTTPStatusError::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::InvalidURL)
            | ImpitPyError(ImpitError::UrlParsingError)
            | ImpitPyError(ImpitError::UrlMissingHostnameError(_))
            | ImpitPyError(ImpitError::UrlProtocolError(_)) => {
                InvalidURL::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::CookieConflict) => {
                CookieConflict::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::StreamError) => StreamError::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::StreamConsumed) => {
                StreamConsumed::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::ResponseNotRead) => {
                ResponseNotRead::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::RequestNotRead) => {
                RequestNotRead::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::StreamClosed) => StreamClosed::new_err(format!("{}", err.0)),
            ImpitPyError(ImpitError::InvalidHeaderName(_)) => {
                LocalProtocolError::new_err(format!("{}", err.0))
            }
            ImpitPyError(ImpitError::InvalidHeaderValue(_)) => {
                LocalProtocolError::new_err(format!("{}", err.0))
            }
            _ => HTTPError::new_err(format!("{}", err.0)),
        }
    }
}
