// Based on https://github.com/square/okhttp/blob/master/okhttp/src/main/java/okhttp3/Cookie.java
// and http://doc.qt.io/qt-5/qnetworkcookie.html.

use std::time::Duration;

/// Represents an HTTP cookie. It has a name, a single value and optional parameters to
/// maintain persistent information across requests.
///
/// Check out the [official specification](https://tools.ietf.org/html/rfc6265).
#[derive(Clone, Debug)]
pub struct Cookie<S> {
    domain: Option<S>,
    expires_at: Option<Duration>,
    is_host_only: bool,
    is_http_only: bool,
    is_persistent: bool,
    is_secure: bool,
    name: S,
    path: Option<S>,
    value: S,
}

impl<S> Cookie<S>
where
    S: AsRef<str>,
{
    /// Creates a new [`Cookie`](Cookie) from the full range of possible parameters.
    ///
    /// # Parameters
    ///
    /// * `domain` - See [`Cookie.domain`](Cookie::domain).
    /// * `expires_at` - See [`Cookie.expires_at`](Cookie::expires_at).
    /// * `is_host_only` - See [`Cookie.is_host_only`](Cookie::is_host_only).
    /// * `is_http_only` - See [`Cookie.is_http_only`](Cookie::is_http_only).
    /// * `is_persistent` - See [`Cookie.is_persistent`](Cookie::is_persistent).
    /// * `is_secure` - See [`Cookie.is_secure`](Cookie::is_secure).
    /// * `name` - See [`Cookie.name`](Cookie::name).
    /// * `path` - See [`Cookie.path`](Cookie::path).
    /// * `value` - See [`Cookie.value`](Cookie::value).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tide::Cookie;
    /// use std::time::{Duration, SystemTime, UNIX_EPOCH};
    ///
    /// let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    /// let one_hour_from_now = now + Duration::from_secs(3600);
    /// let _ = Cookie::new(
    ///     "foo.com",
    ///     one_hour_from_now,
    ///     true,
    ///     true,
    ///     false,
    ///     true,
    ///     "foo",
    ///     "/a-path",
    ///     "bar"
    /// );
    /// ```
    pub fn new<ID, IEA, IP>(
        domain: ID,
        expires_at: IEA,
        is_host_only: bool,
        is_http_only: bool,
        is_persistent: bool,
        is_secure: bool,
        name: S,
        path: IP,
        value: S,
    ) -> Self
    where
        ID: Into<Option<S>>,
        IEA: Into<Option<Duration>>,
        IP: Into<Option<S>>,
    {
        Self {
            domain: domain.into(),
            expires_at: expires_at.into(),
            is_host_only,
            is_http_only,
            is_persistent,
            is_secure,
            name,
            path: path.into(),
            value,
        }
    }

    /// If any, returns the domain associated with this
    /// cookie. E.g.: `Set-Cookie: name=value; Domain=.foo.com`.
    pub fn domain(&self) -> Option<&str> {
        self.domain.as_ref().map(|x| x.as_ref())
    }

    /// If any, returns the time this cookie expires.
    ///
    /// A value less than the current time means that the cookie has been expired.
    pub fn expires_at(&self) -> Option<&Duration> {
        self.expires_at.as_ref()
    }

    /// Converts itself into [`CookieBuilder`](CookieBuilder). Useful
    /// to override any parameter in a flexible and readable way.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tide::Cookie;
    ///
    /// let _ = Cookie::new(None, None, true, true, false, true, "foo", None, "bar")
    ///     .into_builder()
    ///     .name("another-foo")
    ///     .path("/a-path")
    ///     .build();
    /// ```
    pub fn into_builder(self) -> CookieBuilder<S> {
        CookieBuilder {
            domain: self.domain,
            expires_at: self.expires_at,
            is_host_only: self.is_host_only,
            is_http_only: self.is_http_only,
            is_persistent: self.is_persistent,
            is_secure: self.is_secure,
            name: Some(self.name),
            path: self.path,
            value: self.value,
        }
    }

    /// If this cookie should be used for a single and unique domain.
    pub fn is_host_only(&self) -> bool {
        self.is_host_only
    }

    /// If this cookie should be restrained to HTTP APIs only.
    pub fn is_http_only(&self) -> bool {
        self.is_http_only
    }

    /// If this cookie should not expire at the end of the current session.
    pub fn is_persistent(&self) -> bool {
        self.is_persistent
    }

    /// If this cookie should only be transmitted in HTTPS connections.
    pub fn is_secure(&self) -> bool {
        self.is_secure
    }

    /// The name that identifies this cookie. E.g.: `Set-Cookie: name=value`.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// If any, returns the path associated with this
    /// cookie. E.g.: `Set-Cookie: name=value; Path=/`.
    pub fn path(&self) -> Option<&str> {
        self.path.as_ref().map(|x| x.as_ref())
    }

    /// The cookie's value. E.g.: `Set-Cookie: name=value`.
    pub fn value(&self) -> &str {
        self.value.as_ref()
    }
}

/// Provides a handy interface to build a [`Cookie`](Cookie) readably.
#[derive(Clone, Debug)]
pub struct CookieBuilder<S> {
    domain: Option<S>,
    expires_at: Option<Duration>,
    is_host_only: bool,
    is_http_only: bool,
    is_persistent: bool,
    is_secure: bool,
    name: Option<S>,
    path: Option<S>,
    value: S,
}

impl<S> CookieBuilder<S>
where
    S: AsRef<str> + Default,
{
    /// Creates an optioned [`CookieBuilder`](CookieBuilder) that defaults
    /// to conservative parameters.
    ///
    /// * `domain` - None
    /// * `expires_at` - None
    /// * `is_host_only` - true
    /// * `is_http_only` - true
    /// * `is_persistent` - false
    /// * `is_secure` - true
    /// * `path` - None
    /// * `value` - Empty string ("")
    ///
    /// All default parameters can be overwritten and [`name`](CookieBuilder::name) is
    /// the only mandatory method.
    pub fn new() -> Self {
        Self {
            domain: None,
            expires_at: None,
            is_host_only: true,
            is_http_only: true,
            is_persistent: false,
            is_secure: true,
            name: None,
            path: None,
            value: S::default(),
        }
    }

    /// Creates a new [`Cookie`](Cookie) based on the specificated building methods.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tide::CookieBuilder;
    /// use std::time::{Duration, SystemTime, UNIX_EPOCH};
    ///
    /// // Basic cookie
    /// let _ = CookieBuilder::new().name("foo").build();
    ///
    /// // Elaborated cookie
    /// let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    /// let one_hour_from_now = now + Duration::from_secs(3600);
    /// let _ = CookieBuilder::new()
    ///     .domain("foo.com")
    ///     .expires_at(one_hour_from_now)
    ///     .is_persistent(true)
    ///     .name("bar")
    ///     .path("/a-path")
    ///     .build();
    /// ```
    ///
    /// # Assertions
    ///
    /// * The `name` method must be used.
    ///
    /// ```should_panic
    /// use tide::CookieBuilder;
    /// let _ = CookieBuilder::<&str>::new().build();
    /// ```
    pub fn build(self) -> Cookie<S> {
        assert!(self.name.is_some(), "The `name` method must be used");
        Cookie::new(
            self.domain,
            self.expires_at,
            self.is_host_only,
            self.is_http_only,
            self.is_persistent,
            self.is_secure,
            self.name.unwrap(),
            self.path,
            self.value,
        )
    }

    /// See [`Cookie.domain`](Cookie::domain).
    pub fn domain(mut self, domain: S) -> Self {
        self.domain = Some(domain);
        self
    }

    /// See [`Cookie.expires_at`](Cookie::expires_at).
    pub fn expires_at(mut self, expires_at: Duration) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// See [`Cookie.is_host_only`](Cookie::is_host_only).
    pub fn is_host_only(mut self, is_host_only: bool) -> Self {
        self.is_host_only = is_host_only;
        self
    }

    /// See [`Cookie.is_http_only`](Cookie::is_http_only).
    pub fn is_http_only(mut self, is_http_only: bool) -> Self {
        self.is_http_only = is_http_only;
        self
    }

    /// See [`Cookie.is_persistent`](Cookie::is_persistent).
    pub fn is_persistent(mut self, is_persistent: bool) -> Self {
        self.is_persistent = is_persistent;
        self
    }

    /// See [`Cookie.is_secure`](Cookie::is_secure).
    pub fn is_secure(mut self, is_secure: bool) -> Self {
        self.is_secure = is_secure;
        self
    }

    /// See [`Cookie.name`](Cookie::name).
    pub fn name(mut self, name: S) -> Self {
        self.name = Some(name);
        self
    }

    /// See [`Cookie.path`](Cookie::path).
    pub fn path(mut self, path: S) -> Self {
        self.path = Some(path);
        self
    }

    /// See [`Cookie.value`](Cookie::value).
    pub fn value(mut self, value: S) -> Self {
        self.value = value;
        self
    }
}
