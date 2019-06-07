//! Testing framework for tide.
//!
//! ## Examples
//! ```
//! # #![feature(async_await)]
//! let mut app = tide::App::new();
//! app.at("/hello").get(async move |_| "Hello, world!");
//!
//! let mut app = app.test();
//! let res = app.get("/hello")
//!     .send()
//!     .await?;
//!
//! let body = res
//!     .assert_status(200)
//!     .body_string()
//!     .await?;
//!
//! assert_eq!(body, String::from("Hello, world!"));
//! ```

use http_service::Response;
use http_service_mock::{make_server};
use std::error::Error;
use std::fmt;
use std::io;

use super::App;

/// Create an HTTP test.
pub struct HttpTest<State> {
    service: super::Server<State>
}

impl<State: Send + Sync + 'static> HttpTest<State> {
    /// Create a new HTTP Test
    pub fn new(app: App<State>) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let service = app.into_http_service();
        Ok(Self { service })
    }

    /// Make a request on the given HTTP method
    pub fn method(self, method: http::Method, uri: &str) -> Route<State> {
        Route::new(self, method, uri)
    }

    /// Make a request on `GET`
    pub fn get(self, uri: &str) -> Route<State> {
        Route::new(self, http::Method::GET, uri)
    }

    /// Make a request on `HEAD`
    pub fn head(self, uri: &str) -> Route<State> {
        Route::new(self, http::Method::HEAD, uri)
    }

    /// Make a request on `PUT`
    pub fn put(self, uri: &str) -> Route<State> {
        Route::new(self, http::Method::PUT, uri)
    }

    /// Make a request on `POST`
    pub fn post(self, uri: &str) -> Route<State> {
        Route::new(self, http::Method::POST, uri)
    }

    /// Make a request on `DELETE`
    pub fn delete(self, uri: &str) -> Route<State> {
        Route::new(self, http::Method::DELETE, uri)
    }

    /// Make a request on `OPTIONS`
    pub fn options(self, uri: &str) -> Route<State> {
        Route::new(self, http::Method::OPTIONS, uri)
    }

    /// Make a request on `CONNECT`
    pub fn connect(self, uri: &str) -> Route<State> {
        Route::new(self, http::Method::CONNECT, uri)
    }

    /// Make a request on `PATCH`
    pub fn patch(self, uri: &str) -> Route<State> {
        Route::new(self, http::Method::PATCH, uri)
    }

    /// Make a request on `TRACE`
    pub fn trace(self, uri: &str) -> Route<State> {
        Route::new(self, http::Method::TRACE, uri)
    }
}

impl<State> fmt::Debug for HttpTest<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("app", &"...")
            .finish()
    }
}

/// A route as defined by the test module.
#[derive(Debug)]
pub struct Route<State> {
    app: HttpTest<State>,
    req: http::request::Builder,
}

impl<State: Send + Sync + 'static> Route<State> {
    /// Create a mock request
    fn new(app: HttpTest<State>, method: http::Method, uri: &str) -> Self {
        let mut req = http::Request::builder();
        req.method(method);
        req.uri(uri);
        Self { app, req }
    }

    /// Submit the request.
    pub async fn send(
        mut self,
    ) -> Result<HttpTestResponse, Box<dyn Error + Send + Sync + 'static>> {
        let req = self.req.body(http_service::Body::empty())?;
        let mut service = make_server(self.app.service)?;
        let res = service.simulate(req)?;
        Ok(HttpTestResponse { res })
    }
}

/// The response returned from [`Route::send`].
///
/// `Route::send`: struct.Route.html#method.send
#[derive(Debug)]
pub struct HttpTestResponse {
    res: Response,
}

impl HttpTestResponse {
    /// Assert the status code
    pub fn assert_status(self, status: u16) -> Self {
        assert_eq!(self.res.status(), status);
        self
    }

    /// Assert the value of a header.
    pub fn assert_header(self, key: &str, value: &'static str) -> Self {
        let value = http::header::HeaderValue::from_static(value);
        let header = self.res.headers().get(key);
        let header = header.expect("Header did not exist in the map");
        assert_eq!(header, value);
        self
    }

    /// Access the response body.
    pub async fn body(self) -> io::Result<Vec<u8>> {
        let body = self.res.into_body().into_vec().await?;
        Ok(body)
    }

    /// Access the response body as a string.
    pub async fn body_string(self) -> Result<String, Box<dyn Error + Send + Sync + 'static>> {
        let body = self.body().await?;
        Ok(String::from_utf8(body)?)
    }
}
