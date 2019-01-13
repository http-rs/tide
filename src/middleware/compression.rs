use futures::future::FutureObj;

use crate::body::Body;
use crate::{middleware::RequestContext, Middleware, Response};
use http::header::{HeaderValue, ACCEPT_ENCODING, CONTENT_ENCODING};

use std::io::Read;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Encoding {
    Gzip,
    Deflate,
    Identity,
}

impl Encoding {
    fn parse(i: &str) -> Option<Encoding> {
        use self::Encoding::*;
        match i {
            "gzip" => Some(Gzip),
            "deflate" => Some(Deflate),
            "identity" => Some(Identity),
            _ => None,
        }
    }

    fn header_value(self) -> HeaderValue {
        use self::Encoding::*;
        match self {
            Gzip => HeaderValue::from_str("gzip").unwrap(),
            Deflate => HeaderValue::from_str("deflate").unwrap(),
            Identity => HeaderValue::from_str("identity").unwrap(),
        }
    }
}

/// Tide's compression middleware.
/// Will pick preferred supported format from "Accept-Encoding" header
/// and serialize response accordingly
pub struct Compression {
    default: Encoding,
}

impl Compression {
    pub fn new() -> Self {
        Compression {
            default: Encoding::Gzip,
        }
    }

    pub fn with_default(default: Encoding) -> Self {
        Compression { default }
    }

    fn preferred_encoding(headers: &http::HeaderMap) -> Option<Encoding> {
        headers
            .get_all(ACCEPT_ENCODING)
            .iter()
            .filter_map(|hval| hval.to_str().ok())
            .flat_map(|val| val.split(|c| c == ','))
            .filter_map(|encoding| Encoding::parse(encoding))
            .next()
    }

    fn apply(body: &[u8], encoding: Encoding) -> Option<Vec<u8>> {
        let mut buf = Vec::new();
        match encoding {
            Encoding::Gzip => {
                let mut gz = flate2::bufread::GzEncoder::new(body, flate2::Compression::fast());
                gz.read_to_end(&mut buf).ok()?;
            }

            Encoding::Deflate => {
                let mut deflate =
                    flate2::bufread::DeflateEncoder::new(body, flate2::Compression::fast());
                deflate.read_to_end(&mut buf).ok()?;
            }

            _ => return None,
        };
        Some(buf)
    }
}

impl Default for Compression {
    fn default() -> Self {
        Compression::new()
    }
}

/// Picks best ecnoding from request and accordingly set response header and body.
impl<Data: Clone + Send> Middleware<Data> for Compression {
    fn handle<'a>(&'a self, ctx: RequestContext<'a, Data>) -> FutureObj<'a, Response> {
        FutureObj::new(Box::new(
            async move {
                let encoding = Self::preferred_encoding(ctx.req.headers()).unwrap_or(self.default);
                let mut res: crate::Response = await!(ctx.next());
                let body = await!(res.body_mut().read_to_vec()).expect("failed to read reply");
                if let Some(compressed) = Compression::apply(&body, encoding) {
                    *res.body_mut() = Body::from(compressed);
                    res.headers_mut()
                        .append(CONTENT_ENCODING, encoding.header_value());
                } else {
                    *res.body_mut() = Body::from(body);
                }
                res
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::*;
    use crate::{body::Body, endpoint::BoxedEndpoint, middleware::RequestContext, Request};
    use http::status::StatusCode;

    static LOREM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";

    async fn lorem() -> String {
        String::from(LOREM)
    }

    #[test]
    fn identity() {
        let mut req = Request::new(Body::empty());
        req.headers_mut()
            .append(ACCEPT_ENCODING, HeaderValue::from_str("identity").unwrap());
        let mdw = Compression::default();
        let endpoint = BoxedEndpoint::new(lorem);
        let ctx = RequestContext {
            app_data: (),
            req,
            params: None,
            endpoint: &endpoint,
            next_middleware: &[],
        };
        let mut resp = block_on(mdw.handle(ctx));
        assert_eq!(StatusCode::OK, resp.status());
        let body = block_on(resp.body_mut().read_to_vec()).expect("Failed to read body");
        assert_eq!(
            std::str::from_utf8(&body).expect("Failed to convert to UTF-8"),
            LOREM
        );
        assert!(resp.headers().get(CONTENT_ENCODING).is_none());
    }

    #[test]
    fn default_gzip() {
        let req = Request::new(Body::empty());
        let mdw = Compression::default();
        let endpoint = BoxedEndpoint::new(lorem);
        let ctx = RequestContext {
            app_data: (),
            req,
            params: None,
            endpoint: &endpoint,
            next_middleware: &[],
        };
        let mut resp = block_on(mdw.handle(ctx));
        assert_eq!(StatusCode::OK, resp.status());

        let body: Vec<u8> = block_on(resp.body_mut().read_to_vec()).expect("Failed to read body");
        assert_eq!(
            resp.headers()
                .get(CONTENT_ENCODING)
                .expect("No content-encoding header"),
            "gzip"
        );
        let bytes: &[u8] = &body;
        let mut decoder = flate2::read::GzDecoder::new(bytes);
        let mut s = String::new();
        decoder.read_to_string(&mut s).unwrap();
        assert_eq!(s, LOREM);
    }

    fn header_map(accept_encoding: &[&str]) -> hyper::HeaderMap {
        let mut hm = hyper::HeaderMap::new();
        for encoding in accept_encoding {
            hm.insert(ACCEPT_ENCODING, encoding.parse().unwrap());
        }
        hm
    }

    macro_rules! assert_best_encoding {
        ($encodings:expr, $expected:expr) => {
            assert_eq!(
                Compression::preferred_encoding(&header_map($encodings)),
                $expected
            );
        };
    }

    #[test]
    fn preferred_encoding() {
        assert_best_encoding!(&[], None);
        assert_best_encoding!(&["gzip"], Some(Encoding::Gzip));
        assert_best_encoding!(&["unknown"], None);
        assert_best_encoding!(&["gzip,deflate,identity"], Some(Encoding::Gzip));
        assert_best_encoding!(&["unknown", "deflate"], Some(Encoding::Deflate));
    }
}
