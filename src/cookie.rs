// Based on https://github.com/square/okhttp/blob/master/okhttp/src/main/java/okhttp3/Cookie.java
// and http://doc.qt.io/qt-5/qnetworkcookie.html.

use std::time::Duration;

/// Represents an HTTP cookie. It has a name, a single value and optional parameters to
/// maintain persistent information across requests.
///
/// Check out the [official specification](https://tools.ietf.org/html/rfc6265).
#[derive(Clone, Debug)]
pub struct Cookie<S> {
    pub(self) domain: Option<S>,
    pub(self) expires_at: Option<Duration>,
    pub(self) host_only: bool,
    pub(self) http_only: bool,
    pub(self) name: S,
    pub(self) path: Option<S>,
    pub(self) persistent: bool,
    pub(self) secure: bool,
    pub(self) value: S,
}

impl<S> Cookie<S>
where
    S: AsRef<str>,
{
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
    /// use tide::CookieBuilder;
    ///
    /// let _ = CookieBuilder::new()
    ///     .name("foo")
    ///     .path("/a-path")
    ///     .build()
    ///     .into_builder()
    ///     .name("another-foo")
    ///     .build();
    /// ```
    pub fn into_builder(self) -> CookieBuilder<S> {
        CookieBuilder {
            domain: self.domain,
            expires_at: self.expires_at,
            host_only: self.host_only,
            http_only: self.http_only,
            name: Some(self.name),
            path: self.path,
            persistent: self.persistent,
            secure: self.secure,
            value: self.value,
        }
    }

    /// If this cookie should be used for a single and unique domain.
    pub fn host_only(&self) -> bool {
        self.host_only
    }

    /// If this cookie should be restrained to HTTP APIs only.
    pub fn http_only(&self) -> bool {
        self.http_only
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

    /// If this cookie should not expire at the end of the current session.
    pub fn persistent(&self) -> bool {
        self.persistent
    }

    /// If this cookie should only be transmitted in HTTPS connections.
    pub fn secure(&self) -> bool {
        self.secure
    }

    /// The cookie's value. E.g.: `Set-Cookie: name=value`.
    pub fn value(&self) -> &str {
        self.value.as_ref()
    }
}

/// Provides a handy interface to build a [`Cookie`](Cookie) readably.
#[derive(Clone, Debug, Default)]
pub struct CookieBuilder<S> {
    domain: Option<S>,
    expires_at: Option<Duration>,
    host_only: bool,
    http_only: bool,
    name: Option<S>,
    path: Option<S>,
    persistent: bool,
    secure: bool,
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
    /// * `host_only` - true
    /// * `http_only` - true
    /// * `path` - None
    /// * `persistent` - false
    /// * `secure` - true
    /// * `value` - Empty string ("")
    ///
    /// All default parameters can be overwritten and [`name`](CookieBuilder::name) is
    /// the only mandatory method.
    pub fn new() -> Self {
        Self {
            domain: None,
            expires_at: None,
            host_only: true,
            http_only: true,
            name: None,
            path: None,
            persistent: false,
            secure: true,
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
    ///     .name("bar")
    ///     .path("/a-path")
    ///     .persistent(true)
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
        Cookie {
            domain: self.domain,
            expires_at: self.expires_at,
            host_only: self.host_only,
            http_only: self.http_only,
            name: self.name.unwrap(),
            path: self.path,
            persistent: self.persistent,
            secure: self.secure,
            value: self.value,
        }
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

    /// See [`Cookie.host_only`](Cookie::host_only).
    pub fn host_only(mut self, host_only: bool) -> Self {
        self.host_only = host_only;
        self
    }

    /// See [`Cookie.http_only`](Cookie::http_only).
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
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

    /// See [`Cookie.persistent`](Cookie::persistent).
    pub fn persistent(mut self, persistent: bool) -> Self {
        self.persistent = persistent;
        self
    }

    /// See [`Cookie.secure`](Cookie::secure).
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// See [`Cookie.value`](Cookie::value).
    pub fn value(mut self, value: S) -> Self {
        self.value = value;
        self
    }
}
