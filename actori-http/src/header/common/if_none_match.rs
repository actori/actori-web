use crate::header::{EntityTag, IF_NONE_MATCH};

header! {
    /// `If-None-Match` header, defined in
    /// [RFC7232](https://tools.ietf.org/html/rfc7232#section-3.2)
    ///
    /// The `If-None-Match` header field makes the request method conditional
    /// on a recipient cache or origin server either not having any current
    /// representation of the target resource, when the field-value is "*",
    /// or having a selected representation with an entity-tag that does not
    /// match any of those listed in the field-value.
    ///
    /// A recipient MUST use the weak comparison function when comparing
    /// entity-tags for If-None-Match (Section 2.3.2), since weak entity-tags
    /// can be used for cache validation even if there have been changes to
    /// the representation data.
    ///
    /// # ABNF
    ///
    /// ```text
    /// If-None-Match = "*" / 1#entity-tag
    /// ```
    ///
    /// # Example values
    ///
    /// * `"xyzzy"`
    /// * `W/"xyzzy"`
    /// * `"xyzzy", "r2d2xxxx", "c3piozzzz"`
    /// * `W/"xyzzy", W/"r2d2xxxx", W/"c3piozzzz"`
    /// * `*`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use actori_http::Response;
    /// use actori_http::http::header::IfNoneMatch;
    ///
    /// let mut builder = Response::Ok();
    /// builder.set(IfNoneMatch::Any);
    /// ```
    ///
    /// ```rust
    /// use actori_http::Response;
    /// use actori_http::http::header::{IfNoneMatch, EntityTag};
    ///
    /// let mut builder = Response::Ok();
    /// builder.set(
    ///     IfNoneMatch::Items(vec![
    ///         EntityTag::new(false, "xyzzy".to_owned()),
    ///         EntityTag::new(false, "foobar".to_owned()),
    ///         EntityTag::new(false, "bazquux".to_owned()),
    ///     ])
    /// );
    /// ```
    (IfNoneMatch, IF_NONE_MATCH) => {Any / (EntityTag)+}

    test_if_none_match {
        test_header!(test1, vec![b"\"xyzzy\""]);
        test_header!(test2, vec![b"W/\"xyzzy\""]);
        test_header!(test3, vec![b"\"xyzzy\", \"r2d2xxxx\", \"c3piozzzz\""]);
        test_header!(test4, vec![b"W/\"xyzzy\", W/\"r2d2xxxx\", W/\"c3piozzzz\""]);
        test_header!(test5, vec![b"*"]);
    }
}

#[cfg(test)]
mod tests {
    use super::IfNoneMatch;
    use crate::header::{EntityTag, Header, IF_NONE_MATCH};
    use crate::test::TestRequest;

    #[test]
    fn test_if_none_match() {
        let mut if_none_match: Result<IfNoneMatch, _>;

        let req = TestRequest::with_header(IF_NONE_MATCH, "*").finish();
        if_none_match = Header::parse(&req);
        assert_eq!(if_none_match.ok(), Some(IfNoneMatch::Any));

        let req =
            TestRequest::with_header(IF_NONE_MATCH, &b"\"foobar\", W/\"weak-etag\""[..])
                .finish();

        if_none_match = Header::parse(&req);
        let mut entities: Vec<EntityTag> = Vec::new();
        let foobar_etag = EntityTag::new(false, "foobar".to_owned());
        let weak_etag = EntityTag::new(true, "weak-etag".to_owned());
        entities.push(foobar_etag);
        entities.push(weak_etag);
        assert_eq!(if_none_match.ok(), Some(IfNoneMatch::Items(entities)));
    }
}
