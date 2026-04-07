class ImpitError extends Error { constructor(msg) { super(msg); this.name = this.constructor.name; } }
class HTTPError extends ImpitError {}
class RequestError extends ImpitError {}
class TransportError extends RequestError {}
class TimeoutError extends TransportError {}
class ConnectTimeout extends TimeoutError {}
class ReadTimeout extends TimeoutError {}
class WriteTimeout extends TimeoutError {}
class PoolTimeout extends TimeoutError {}
class NetworkError extends TransportError {}
class ConnectError extends NetworkError {}
class ReadError extends NetworkError {}
class WriteError extends NetworkError {}
class CloseError extends NetworkError {}
class ProtocolError extends TransportError {}
class LocalProtocolError extends ProtocolError {}
class RemoteProtocolError extends ProtocolError {}
class ProxyError extends TransportError {}
class ProxyTunnelError extends ProxyError {
    constructor(msg) {
        super(msg);
        const match = msg.match(/with status code (\d+)/);
        this.status = match ? Number(match[1]) : undefined;
    }
}
class ProxyAuthRequired extends ProxyError {}
class UnsupportedProtocol extends TransportError {}
class DecodingError extends RequestError {}
class TooManyRedirects extends RequestError {}
class HTTPStatusError extends HTTPError {}
class InvalidURL extends ImpitError {}
class CookieConflict extends ImpitError {}
class StreamError extends ImpitError {}
class StreamConsumed extends StreamError {}
class ResponseNotRead extends StreamError {}
class RequestNotRead extends StreamError {}
class StreamClosed extends StreamError {}

const errorClassMap = {
    HTTPError,
    RequestError,
    TransportError,
    TimeoutException: TimeoutError,
    ConnectTimeout,
    ReadTimeout,
    WriteTimeout,
    PoolTimeout,
    NetworkError,
    ConnectError,
    ReadError,
    WriteError,
    CloseError,
    ProtocolError,
    LocalProtocolError,
    RemoteProtocolError,
    ProxyError,
    ProxyTunnelError,
    ProxyAuthRequired,
    UnsupportedProtocol,
    DecodingError,
    TooManyRedirects,
    HTTPStatusError,
    InvalidURL,
    CookieConflict,
    StreamError,
    StreamConsumed,
    ResponseNotRead,
    RequestNotRead,
    StreamClosed,
};

function rethrowNativeError(err) {
    const match = err.message?.match(/^(\w+): ([\s\S]*)$/);
    if (match) {
        const [, code, message] = match;
        const ErrorClass = errorClassMap[code];
        if (ErrorClass) {
            throw new ErrorClass(message, { cause: err.cause });
        }
    }
    throw err;
}

module.exports = {
    rethrowNativeError,
    ImpitError,
    HTTPError,
    RequestError,
    TransportError,
    TimeoutError,
    ConnectTimeout,
    ReadTimeout,
    WriteTimeout,
    PoolTimeout,
    NetworkError,
    ConnectError,
    ReadError,
    WriteError,
    CloseError,
    ProtocolError,
    LocalProtocolError,
    RemoteProtocolError,
    ProxyError,
    ProxyTunnelError,
    ProxyAuthRequired,
    UnsupportedProtocol,
    DecodingError,
    TooManyRedirects,
    HTTPStatusError,
    InvalidURL,
    CookieConflict,
    StreamError,
    StreamConsumed,
    ResponseNotRead,
    RequestNotRead,
    StreamClosed,
};
