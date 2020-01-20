use crate::header::CONTENT_TYPE;
use mime::Mime;

header! {
    /// `Content-Type` header, defined in
    /// [RFC7231](http://tools.ietf.org/html/rfc7231#section-3.1.1.5)
    ///
    /// The `Content-Type` header field indicates the media type of the
    /// associated representation: either the representation enclosed in the
    /// message payload or the selected representation, as determined by the
    /// message semantics.  The indicated media type defines both the data
    /// format and how that data is intended to be processed by a recipient,
    /// within the scope of the received message semantics, after any content
    /// codings indicated by Content-Encoding are decoded.
    ///
    /// Although the `mime` crate allows the mime options to be any slice, this crate
    /// forces the use of Vec. This is to make sure the same header can't have more than 1 type. If
    /// this is an issue, it's possible to implement `Header` on a custom struct.
    ///
    /// # ABNF
    ///
    /// ```text
    /// Content-Type = media-type
    /// ```
    ///
    /// # Example values
    ///
    /// * `text/html; charset=utf-8`
    /// * `application/json`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use actori_http::Response;
    /// use actori_http::http::header::ContentType;
    ///
    /// # fn main() {
    /// let mut builder = Response::Ok();
    /// builder.set(
    ///     ContentType::json()
    /// );
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate mime;
    /// # extern crate actori_http;
    /// use mime::TEXT_HTML;
    /// use actori_http::Response;
    /// use actori_http::http::header::ContentType;
    ///
    /// # fn main() {
    /// let mut builder = Response::Ok();
    /// builder.set(
    ///     ContentType(TEXT_HTML)
    /// );
    /// # }
    /// ```
    (ContentType, CONTENT_TYPE) => [Mime]

    test_content_type {
        test_header!(
            test1,
            vec![b"text/html"],
            Some(HeaderField(mime::TEXT_HTML)));
    }
}

impl ContentType {
    /// A constructor  to easily create a `Content-Type: application/json`
    /// header.
    #[inline]
    pub fn json() -> ContentType {
        ContentType(mime::APPLICATION_JSON)
    }

    /// A constructor  to easily create a `Content-Type: text/plain;
    /// charset=utf-8` header.
    #[inline]
    pub fn plaintext() -> ContentType {
        ContentType(mime::TEXT_PLAIN_UTF_8)
    }

    /// A constructor  to easily create a `Content-Type: text/html` header.
    #[inline]
    pub fn html() -> ContentType {
        ContentType(mime::TEXT_HTML)
    }

    /// A constructor  to easily create a `Content-Type: text/xml` header.
    #[inline]
    pub fn xml() -> ContentType {
        ContentType(mime::TEXT_XML)
    }

    /// A constructor  to easily create a `Content-Type:
    /// application/www-form-url-encoded` header.
    #[inline]
    pub fn form_url_encoded() -> ContentType {
        ContentType(mime::APPLICATION_WWW_FORM_URLENCODED)
    }
    /// A constructor  to easily create a `Content-Type: image/jpeg` header.
    #[inline]
    pub fn jpeg() -> ContentType {
        ContentType(mime::IMAGE_JPEG)
    }

    /// A constructor  to easily create a `Content-Type: image/png` header.
    #[inline]
    pub fn png() -> ContentType {
        ContentType(mime::IMAGE_PNG)
    }

    /// A constructor  to easily create a `Content-Type:
    /// application/octet-stream` header.
    #[inline]
    pub fn octet_stream() -> ContentType {
        ContentType(mime::APPLICATION_OCTET_STREAM)
    }
}

impl Eq for ContentType {}
