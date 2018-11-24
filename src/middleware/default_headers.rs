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
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: IntoHeaderName,
        HeaderValue: HttpTryFrom<V>,
    {
        let value = HeaderValue::try_from(value)
            .map_err(Into::into)
            .expect("Cannot create default header");

        self.headers.append(key, value);

        self
    }
}

impl<Data> Middleware<Data> for DefaultHeaders {
    fn response<'a>(
        &'a self,
        data: &'a mut Data,
        head: &'a Head,
        resp: &'a mut Response,
    ) -> FutureObj<'a, ()> {
        FutureObj::new(Box::new(
            async move {
                let headers = resp.headers_mut();
                for (key, value) in self.headers.iter() {
                    headers.entry(key).unwrap().or_insert_with(|| value.clone());
                }
            },
        ))
    }
}
