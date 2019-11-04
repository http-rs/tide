/// A raw HTTP body.
///
/// This part can be part of either a `Request` or `Response`.
pub struct Body {
    body: http_service::Body,
}

impl Body {
    /// Create a new empty body
    pub fn empty() -> Self {
        Self {
            body: http_service::Body::empty(),
        }
    }

    // TODO: impl from_reader
}
