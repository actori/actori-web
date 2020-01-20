use std::io;

use actori_connect::resolver::ResolveError;
use derive_more::{Display, From};

#[cfg(feature = "openssl")]
use actori_connect::ssl::openssl::{HandshakeError, SslError};

use crate::error::{Error, ParseError, ResponseError};
use crate::http::{Error as HttpError, StatusCode};

/// A set of errors that can occur while connecting to an HTTP host
#[derive(Debug, Display, From)]
pub enum ConnectError {
    /// SSL feature is not enabled
    #[display(fmt = "SSL is not supported")]
    SslIsNotSupported,

    /// SSL error
    #[cfg(feature = "openssl")]
    #[display(fmt = "{}", _0)]
    SslError(SslError),

    /// SSL Handshake error
    #[cfg(feature = "openssl")]
    #[display(fmt = "{}", _0)]
    SslHandshakeError(String),

    /// Failed to resolve the hostname
    #[display(fmt = "Failed resolving hostname: {}", _0)]
    Resolver(ResolveError),

    /// No dns records
    #[display(fmt = "No dns records found for the input")]
    NoRecords,

    /// Http2 error
    #[display(fmt = "{}", _0)]
    H2(h2::Error),

    /// Connecting took too long
    #[display(fmt = "Timeout out while establishing connection")]
    Timeout,

    /// Connector has been disconnected
    #[display(fmt = "Internal error: connector has been disconnected")]
    Disconnected,

    /// Unresolved host name
    #[display(fmt = "Connector received `Connect` method with unresolved host")]
    Unresolverd,

    /// Connection io error
    #[display(fmt = "{}", _0)]
    Io(io::Error),
}

impl From<actori_connect::ConnectError> for ConnectError {
    fn from(err: actori_connect::ConnectError) -> ConnectError {
        match err {
            actori_connect::ConnectError::Resolver(e) => ConnectError::Resolver(e),
            actori_connect::ConnectError::NoRecords => ConnectError::NoRecords,
            actori_connect::ConnectError::InvalidInput => panic!(),
            actori_connect::ConnectError::Unresolverd => ConnectError::Unresolverd,
            actori_connect::ConnectError::Io(e) => ConnectError::Io(e),
        }
    }
}

#[cfg(feature = "openssl")]
impl<T: std::fmt::Debug> From<HandshakeError<T>> for ConnectError {
    fn from(err: HandshakeError<T>) -> ConnectError {
        ConnectError::SslHandshakeError(format!("{:?}", err))
    }
}

#[derive(Debug, Display, From)]
pub enum InvalidUrl {
    #[display(fmt = "Missing url scheme")]
    MissingScheme,
    #[display(fmt = "Unknown url scheme")]
    UnknownScheme,
    #[display(fmt = "Missing host name")]
    MissingHost,
    #[display(fmt = "Url parse error: {}", _0)]
    HttpError(http::Error),
}

/// A set of errors that can occur during request sending and response reading
#[derive(Debug, Display, From)]
pub enum SendRequestError {
    /// Invalid URL
    #[display(fmt = "Invalid URL: {}", _0)]
    Url(InvalidUrl),
    /// Failed to connect to host
    #[display(fmt = "Failed to connect to host: {}", _0)]
    Connect(ConnectError),
    /// Error sending request
    Send(io::Error),
    /// Error parsing response
    Response(ParseError),
    /// Http error
    #[display(fmt = "{}", _0)]
    Http(HttpError),
    /// Http2 error
    #[display(fmt = "{}", _0)]
    H2(h2::Error),
    /// Response took too long
    #[display(fmt = "Timeout out while waiting for response")]
    Timeout,
    /// Tunnels are not supported for http2 connection
    #[display(fmt = "Tunnels are not supported for http2 connection")]
    TunnelNotSupported,
    /// Error sending request body
    Body(Error),
}

/// Convert `SendRequestError` to a server `Response`
impl ResponseError for SendRequestError {
    fn status_code(&self) -> StatusCode {
        match *self {
            SendRequestError::Connect(ConnectError::Timeout) => {
                StatusCode::GATEWAY_TIMEOUT
            }
            SendRequestError::Connect(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// A set of errors that can occur during freezing a request
#[derive(Debug, Display, From)]
pub enum FreezeRequestError {
    /// Invalid URL
    #[display(fmt = "Invalid URL: {}", _0)]
    Url(InvalidUrl),
    /// Http error
    #[display(fmt = "{}", _0)]
    Http(HttpError),
}

impl From<FreezeRequestError> for SendRequestError {
    fn from(e: FreezeRequestError) -> Self {
        match e {
            FreezeRequestError::Url(e) => e.into(),
            FreezeRequestError::Http(e) => e.into(),
        }
    }
}
