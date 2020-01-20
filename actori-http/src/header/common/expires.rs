use crate::header::{HttpDate, EXPIRES};

header! {
    /// `Expires` header, defined in [RFC7234](http://tools.ietf.org/html/rfc7234#section-5.3)
    ///
    /// The `Expires` header field gives the date/time after which the
    /// response is considered stale.
    ///
    /// The presence of an Expires field does not imply that the original
    /// resource will change or cease to exist at, before, or after that
    /// time.
    ///
    /// # ABNF
    ///
    /// ```text
    /// Expires = HTTP-date
    /// ```
    ///
    /// # Example values
    /// * `Thu, 01 Dec 1994 16:00:00 GMT`
    ///
    /// # Example
    ///
    /// ```rust
    /// use actori_http::Response;
    /// use actori_http::http::header::Expires;
    /// use std::time::{SystemTime, Duration};
    ///
    /// let mut builder = Response::Ok();
    /// let expiration = SystemTime::now() + Duration::from_secs(60 * 60 * 24);
    /// builder.set(Expires(expiration.into()));
    /// ```
    (Expires, EXPIRES) => [HttpDate]

    test_expires {
        // Test case from RFC
        test_header!(test1, vec![b"Thu, 01 Dec 1994 16:00:00 GMT"]);
    }
}
