use futures::future::FutureObj;

use http::{
    header::{HeaderValue, IntoHeaderName},
    HeaderMap, HttpTryFrom,
};

use crate::{middleware::RequestContext, Middleware, Response};

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

impl<Data: Clone + Send> Middleware<Data> for DefaultHeaders {
    fn handle<'a>(&'a self, ctx: RequestContext<'a, Data>) -> FutureObj<'a, Response> {
        FutureObj::new(Box::new(
            async move {
                let mut res = await!(ctx.next());

                let headers = res.headers_mut();
                for (key, value) in self.headers.iter() {
                    headers.entry(key).unwrap().or_insert_with(|| value.clone());
                }
                res
            },
        ))
    }
}
