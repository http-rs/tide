//! Crate that provides helpers and/or middlewares for Tide
//! related to compression.

#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations,
    missing_docs
)]

pub use accept_encoding::Encoding;
use async_compression::stream;
use futures::future::BoxFuture;
use http::{header::CONTENT_ENCODING, status::StatusCode, HeaderMap};
use http_service::{Body, Request};
use tide::{
    middleware::{Middleware, Next},
    response::IntoResponse,
    Context, Error, Response,
};

/// Encode settings for the compression middleware.
///
/// This can be modified in the case that you want more control over the speed or quality of compression.
///
/// For more information on how to configure each of these settings, see the async-compression crate.
#[derive(Debug)]
pub struct EncodeSettings {
    /// Settings for gzip compression.
    pub gzip: async_compression::flate2::Compression,
    /// Settings for deflate compression.
    pub deflate: async_compression::flate2::Compression,
    /// Settings for brotli compression. Ranges from 0-11. (default: `11`)
    pub brotli: u32,
    /// Settings for zstd compression. Ranges from 1-21. (default: `3`)
    pub zstd: i32,
}

impl Default for EncodeSettings {
    fn default() -> Self {
        Self {
            gzip: Default::default(),
            deflate: Default::default(),
            brotli: 11,
            zstd: 3,
        }
    }
}

/// Middleware for automatically handling outgoing response compression.
///
/// This middleware currently supports HTTP compression using `gzip`, `deflate`, `br`, and `zstd`.
#[derive(Debug)]
pub struct Compression {
    default_encoding: Encoding,
    settings: EncodeSettings,
}

impl Default for Compression {
    fn default() -> Self {
        Self::new()
    }
}

impl Compression {
    /// Creates a new Compression middleware. The default encoding is [`Encoding::Identity`] (no encoding).
    pub fn new() -> Self {
        Self {
            default_encoding: Encoding::Identity,
            settings: Default::default(),
        }
    }

    /// Creates a new Compression middleware with a provided default encoding.
    ///
    /// This encoding will be selected if the client has not set the `Accept-Encoding` header or `*` is set as the most preferred encoding.
    pub fn with_default(default_encoding: Encoding) -> Self {
        Self {
            default_encoding,
            settings: Default::default(),
        }
    }

    /// Accesses a mutable handle to this middleware's [`EncodeSettings`].
    ///
    /// This will allow you to configure this middleware's settings.
    pub fn settings_mut(&mut self) -> &mut EncodeSettings {
        &mut self.settings
    }

    fn preferred_encoding(&self, headers: &HeaderMap) -> Result<Encoding, Error> {
        let encoding = match accept_encoding::parse(headers) {
            Ok(encoding) => encoding,
            Err(_) => return Err(Error::from(StatusCode::BAD_REQUEST)),
        };
        Ok(encoding.unwrap_or(self.default_encoding))
    }

    /// Consumes the response and returns an encoded version of it.
    fn encode(&self, mut res: Response, encoding: Encoding) -> Response {
        if res.headers().get(CONTENT_ENCODING).is_some() || encoding == Encoding::Identity {
            return res; // avoid double-encoding a given response
        }
        let body = std::mem::replace(res.body_mut(), Body::empty());
        match encoding {
            Encoding::Gzip => {
                let stream = stream::GzipEncoder::new(body, self.settings.gzip);
                *res.body_mut() = Body::from_stream(stream);
            }
            Encoding::Deflate => {
                let stream = stream::ZlibEncoder::new(body, self.settings.deflate);
                *res.body_mut() = Body::from_stream(stream);
            }
            Encoding::Brotli => {
                let stream = stream::BrotliEncoder::new(body, self.settings.brotli);
                *res.body_mut() = Body::from_stream(stream);
            }
            Encoding::Zstd => {
                let stream = stream::ZstdEncoder::new(body, self.settings.zstd);
                *res.body_mut() = Body::from_stream(stream);
            }
            Encoding::Identity => unreachable!(),
        };
        res.headers_mut()
            .append(CONTENT_ENCODING, encoding.to_header_value());
        res
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for Compression {
    fn handle<'a>(&'a self, cx: Context<State>, next: Next<'a, State>) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            let encoding = match self.preferred_encoding(cx.headers()) {
                Ok(encoding) => encoding,
                Err(e) => return e.into_response(),
            };
            let res = next.run(cx).await;
            self.encode(res, encoding)
        })
    }
}

/// Middleware for handling incoming request decompression.
///
/// This middleware currently supports HTTP decompression under the `gzip`, `deflate`, `br`, and `zstd` algorithms.
#[derive(Debug, Default)]
pub struct Decompression {}

impl Decompression {
    /// Creates a new Decompression middleware.
    pub fn new() -> Self {
        Self {}
    }

    fn parse_encoding(s: &str) -> Result<Encoding, Error> {
        match s {
            "gzip" => Ok(Encoding::Gzip),
            "deflate" => Ok(Encoding::Deflate),
            "br" => Ok(Encoding::Brotli),
            "zstd" => Ok(Encoding::Zstd),
            "identity" => Ok(Encoding::Identity),
            _ => Err(Error::from(StatusCode::UNSUPPORTED_MEDIA_TYPE)),
        }
    }

    fn decode(&self, req: &mut Request) -> Result<(), Error> {
        let encodings = if let Some(hval) = req.headers().get(CONTENT_ENCODING) {
            let hval = match hval.to_str() {
                Ok(hval) => hval,
                Err(_) => return Err(Error::from(StatusCode::BAD_REQUEST)),
            };
            hval.split(',')
                .map(str::trim)
                .rev() // apply decodings in reverse order
                .map(Decompression::parse_encoding)
                .collect::<Result<Vec<Encoding>, Error>>()?
        } else {
            return Ok(());
        };

        for encoding in encodings {
            match encoding {
                Encoding::Gzip => {
                    let body = std::mem::replace(req.body_mut(), Body::empty());
                    let stream = stream::GzipDecoder::new(body);
                    *req.body_mut() = Body::from_stream(stream);
                }
                Encoding::Deflate => {
                    let body = std::mem::replace(req.body_mut(), Body::empty());
                    let stream = stream::ZlibDecoder::new(body);
                    *req.body_mut() = Body::from_stream(stream);
                }
                Encoding::Brotli => {
                    let body = std::mem::replace(req.body_mut(), Body::empty());
                    let stream = stream::BrotliDecoder::new(body);
                    *req.body_mut() = Body::from_stream(stream);
                }
                Encoding::Zstd => {
                    let body = std::mem::replace(req.body_mut(), Body::empty());
                    let stream = stream::ZstdDecoder::new(body);
                    *req.body_mut() = Body::from_stream(stream);
                }
                Encoding::Identity => (),
            }
        }

        // strip the content-encoding header
        req.headers_mut().remove(CONTENT_ENCODING).unwrap();

        Ok(())
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for Decompression {
    fn handle<'a>(
        &'a self,
        mut cx: Context<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            match self.decode(cx.request_mut()) {
                Ok(_) => (),
                Err(e) => return e.into_response(),
            };
            next.run(cx).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_compression::flate2;
    use bytes::Bytes;
    use futures::{
        executor::{block_on, block_on_stream},
        stream::StreamExt,
    };
    use http::header::ACCEPT_ENCODING;
    use http_service::Body;
    use http_service_mock::make_server;

    async fn lorem_ipsum(_cx: Context<()>) -> String {
        String::from(r#"
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aliquam rutrum et risus sed egestas. Maecenas dapibus enim a posuere
            semper. Cras venenatis et turpis quis aliquam. Suspendisse eget risus in libero tristique consectetur. Ut ut risus cursus, scelerisque
            enim ac, tempus tellus. Vestibulum ac porta felis. Aenean fringilla posuere felis, in blandit enim tristique ut. Sed elementum iaculis
            enim eu commodo.
        "#)
    }

    fn lorem_ipsum_bytes() -> Vec<u8> {
        String::from(r#"
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aliquam rutrum et risus sed egestas. Maecenas dapibus enim a posuere
            semper. Cras venenatis et turpis quis aliquam. Suspendisse eget risus in libero tristique consectetur. Ut ut risus cursus, scelerisque
            enim ac, tempus tellus. Vestibulum ac porta felis. Aenean fringilla posuere felis, in blandit enim tristique ut. Sed elementum iaculis
            enim eu commodo.
        "#).into_bytes()
    }

    // Echoes the request body in bytes.
    async fn echo_bytes(mut cx: Context<()>) -> Vec<u8> {
        cx.body_bytes().await.unwrap()
    }

    // Generates the app.
    fn app() -> tide::App<()> {
        let mut app = tide::App::new();
        app.at("/").get(lorem_ipsum);
        app.at("/echo").post(echo_bytes);
        app.middleware(Compression::new());
        app.middleware(Decompression::new());
        app
    }

    // Generates a response given a string that represents the Accept-Encoding header value.
    fn get_encoded_response(hval: &str) -> Response {
        let app = app();
        let mut server = make_server(app.into_http_service()).unwrap();
        let req = http::Request::get("/")
            .header(ACCEPT_ENCODING, hval)
            .body(Body::empty())
            .unwrap();
        server.simulate(req).unwrap()
    }

    // Generates a decoded response given a request body and the header value representing its encoding.
    fn get_decoded_response(body: Body, hval: &str) -> Response {
        let app = app();
        let mut server = make_server(app.into_http_service()).unwrap();
        let req = http::Request::post("/echo")
            .header(CONTENT_ENCODING, hval)
            .body(body)
            .unwrap();
        server.simulate(req).unwrap()
    }

    #[test]
    fn compressed_gzip_response() {
        let res = get_encoded_response("gzip");
        assert_eq!(res.status(), 200);
        let body = res.into_body();
        let stream = stream::GzipDecoder::new(body);
        let decompressed_body: Vec<u8> = block_on_stream(stream)
            .map(Result::unwrap)
            .flatten()
            .collect();
        let lorem_ipsum = lorem_ipsum_bytes();
        assert_eq!(decompressed_body, lorem_ipsum);
    }

    #[test]
    fn compressed_deflate_response() {
        let res = get_encoded_response("deflate");
        assert_eq!(res.status(), 200);
        let body = res.into_body();
        let stream = stream::ZlibDecoder::new(body);
        let decompressed_body: Vec<u8> = block_on_stream(stream)
            .map(Result::unwrap)
            .flatten()
            .collect();
        let lorem_ipsum = lorem_ipsum_bytes();
        assert_eq!(decompressed_body, lorem_ipsum);
    }

    #[test]
    fn compressed_brotli_response() {
        let res = get_encoded_response("br");
        assert_eq!(res.status(), 200);
        let body = res.into_body();
        let stream = stream::BrotliDecoder::new(body);
        let decompressed_body: Vec<u8> = block_on_stream(stream)
            .map(Result::unwrap)
            .flatten()
            .collect();
        let lorem_ipsum = lorem_ipsum_bytes();
        assert_eq!(decompressed_body, lorem_ipsum);
    }

    #[test]
    fn compressed_zstd_response() {
        let res = get_encoded_response("zstd");
        assert_eq!(res.status(), 200);
        let body = res.into_body();
        let stream = stream::ZstdDecoder::new(body);
        let decompressed_body: Vec<u8> = block_on_stream(stream)
            .map(Result::unwrap)
            .flatten()
            .collect();
        let lorem_ipsum = lorem_ipsum_bytes();
        assert_eq!(decompressed_body, lorem_ipsum);
    }

    #[test]
    fn decompressed_gzip_response() {
        let lorem_ipsum = lorem_ipsum_bytes();
        let req_body = Body::from_stream(stream::GzipEncoder::new(
            futures::stream::iter(vec![lorem_ipsum])
                .map(Bytes::from)
                .map(Ok),
            flate2::Compression::default(),
        ));
        let res = get_decoded_response(req_body, "gzip");
        let body = block_on(res.into_body().into_vec()).unwrap();
        let lorem_ipsum = lorem_ipsum_bytes();
        assert_eq!(body, lorem_ipsum);
    }

    #[test]
    fn decompressed_deflate_response() {
        let lorem_ipsum = lorem_ipsum_bytes();
        let req_body = Body::from_stream(stream::ZlibEncoder::new(
            futures::stream::iter(vec![lorem_ipsum])
                .map(Bytes::from)
                .map(Ok),
            flate2::Compression::default(),
        ));
        let res = get_decoded_response(req_body, "deflate");
        let body = block_on(res.into_body().into_vec()).unwrap();
        let lorem_ipsum = lorem_ipsum_bytes();
        assert_eq!(body, lorem_ipsum);
    }

    #[test]
    fn decompressed_brotli_response() {
        let lorem_ipsum = lorem_ipsum_bytes();
        let req_body = Body::from_stream(stream::BrotliEncoder::new(
            futures::stream::iter(vec![lorem_ipsum])
                .map(Bytes::from)
                .map(Ok),
            11,
        ));
        let res = get_decoded_response(req_body, "br");
        let body = block_on(res.into_body().into_vec()).unwrap();
        let lorem_ipsum = lorem_ipsum_bytes();
        assert_eq!(body, lorem_ipsum);
    }

    #[test]
    fn decompressed_zstd_response() {
        let lorem_ipsum = lorem_ipsum_bytes();
        let req_body = Body::from_stream(stream::ZstdEncoder::new(
            futures::stream::iter(vec![lorem_ipsum])
                .map(Bytes::from)
                .map(Ok),
            3,
        ));
        let res = get_decoded_response(req_body, "zstd");
        let body = block_on(res.into_body().into_vec()).unwrap();
        let lorem_ipsum = lorem_ipsum_bytes();
        assert_eq!(body, lorem_ipsum);
    }
}
