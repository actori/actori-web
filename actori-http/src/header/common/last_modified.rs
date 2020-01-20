use crate::header::{HttpDate, LAST_MODIFIED};

header! {
    /// `Last-Modified` header, defined in
    /// [RFC7232](http://tools.ietf.org/html/rfc7232#section-2.2)
    ///
    /// The `Last-Modified` header field in a response provides a timestamp
    /// indicating the date and time at which the origin server believes the
    /// selected representation was last modified, as determined at the
    /// conclusion of handling the request.
    ///
    /// # ABNF
    ///
    /// ```text
    /// Expires = HTTP-date
    /// ```
    ///
    /// # Example values
    ///
    /// * `Sat, 29 Oct 1994 19:43:31 GMT`
    ///
    /// # Example
    ///
    /// ```rust
    /// use actori_http::Response;
    /// use actori_http::http::header::LastModified;
    /// use std::time::{SystemTime, Duration};
    ///
    /// let mut builder = Response::Ok();
    /// let modified = SystemTime::now() - Duration::from_secs(60 * 60 * 24);
    /// builder.set(LastModified(modified.into()));
    /// ```
    (LastModified, LAST_MODIFIED) => [HttpDate]

        test_last_modified {
            // Test case from RFC
            test_header!(test1, vec![b"Sat, 29 Oct 1994 19:43:31 GMT"]);}
}
