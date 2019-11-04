use crate::{error::Error, Request};
use http::StatusCode;
use serde::Deserialize;

/// An extension trait for `Request`, providing query string deserialization.
///
/// # Example
///
/// Turning the query parameters into a `HashMap`:
///
/// ```
/// # use std::collections::HashMap;
/// use tide::querystring::RequestExt;
///
/// let mut app = tide::Server::new();
/// app.at("/").get(|cx: tide::Request<()>| async move {
///     let map: HashMap<String, String> = cx.url_query().unwrap();
///     format!("{:?}", map)
/// });
/// ```
pub trait RequestExt<'de> {
    fn url_query<T: Deserialize<'de>>(&'de self) -> Result<T, Error>;
}

impl<'de, State> RequestExt<'de> for Request<State> {
    #[inline]
    fn url_query<T: Deserialize<'de>>(&'de self) -> Result<T, Error> {
        let query = self.uri().query();

        if query.is_none() {
            return Err(Error::from(StatusCode::BAD_REQUEST));
        }

        Ok(serde_qs::from_str(query.unwrap()).map_err(|_| Error::from(StatusCode::BAD_REQUEST))?)
    }
}
