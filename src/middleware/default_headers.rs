use futures::future::FutureObj;

use http::{
    header::{HeaderValue, IntoHeaderName},
    HeaderMap, HttpTryFrom,
};

use crate::{head::Head, Middleware, Response};

#[derive(Clone, Default)]
pub struct DefaultHeaders {
    headers: HeaderMap,
}

impl DefaultHeaders {
    pub fn new() -> DefaultHeaders {
        DefaultHeaders::default()
    }

    #[inline]
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::match_wild_err_arm))]
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: IntoHeaderName,
        HeaderValue: HttpTryFrom<V>,
    {
        match HeaderValue::try_from(value) {
            Ok(value) => {
                self.headers.append(key, value);
            }
            Err(_) => {
                panic!("Cannot create default header. Please check your default header values.")
            }
        }

        self
    }
}

impl<Data> Middleware<Data> for DefaultHeaders {
    fn response(
        &self,
        data: &mut Data,
        head: &Head,
        mut resp: Response,
    ) -> FutureObj<'static, Response> {
        let headers = resp.headers_mut();
        for (key, value) in self.headers.iter() {
            headers.entry(key).unwrap().or_insert_with(|| value.clone());
        }
        FutureObj::new(Box::new(async { resp }))
    }
}
