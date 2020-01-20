//! Error and Result module
use actori_web::error::{ParseError, PayloadError};
use actori_web::http::StatusCode;
use actori_web::ResponseError;
use derive_more::{Display, From};

/// A set of errors that can occur during parsing multipart streams
#[derive(Debug, Display, From)]
pub enum MultipartError {
    /// Content-Type header is not found
    #[display(fmt = "No Content-type header found")]
    NoContentType,
    /// Can not parse Content-Type header
    #[display(fmt = "Can not parse Content-Type header")]
    ParseContentType,
    /// Multipart boundary is not found
    #[display(fmt = "Multipart boundary is not found")]
    Boundary,
    /// Nested multipart is not supported
    #[display(fmt = "Nested multipart is not supported")]
    Nested,
    /// Multipart stream is incomplete
    #[display(fmt = "Multipart stream is incomplete")]
    Incomplete,
    /// Error during field parsing
    #[display(fmt = "{}", _0)]
    Parse(ParseError),
    /// Payload error
    #[display(fmt = "{}", _0)]
    Payload(PayloadError),
    /// Not consumed
    #[display(fmt = "Multipart stream is not consumed")]
    NotConsumed,
}

/// Return `BadRequest` for `MultipartError`
impl ResponseError for MultipartError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actori_web::HttpResponse;

    #[test]
    fn test_multipart_error() {
        let resp: HttpResponse = MultipartError::Boundary.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
