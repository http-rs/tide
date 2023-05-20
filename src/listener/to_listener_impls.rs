#[cfg(unix)]
use super::UnixListener;
use super::{ConcurrentListener, FailoverListener, ParsedListener, TcpListener, ToListener};
use crate::http::url::Url;
use async_std::io;
use std::net::ToSocketAddrs;

impl ToListener for Url {
    type Listener = ParsedListener;

    fn to_listener(self) -> io::Result<Self::Listener> {
        match self.scheme() {
            "http+unix" => {
                #[cfg(unix)]
                {
                    let path = std::path::PathBuf::from(format!(
                        "{}{}",
                        self.domain().unwrap_or_default(),
                        self.path()
                    ));

                    Ok(ParsedListener::Unix(UnixListener::from_path(path)))
                }

                #[cfg(not(unix))]
                {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Unix sockets not supported on this platform",
                    ))
                }
            }

            "tcp" | "http" => Ok(ParsedListener::Tcp(TcpListener::from_addrs(
                self.socket_addrs(|| Some(80))?,
            ))),

            "tls" | "ssl" | "https" => Err(io::Error::new(
                io::ErrorKind::Other,
                "parsing TLS listeners not supported yet",
            )),

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unrecognized url scheme",
            )),
        }
    }
}

impl ToListener for String {
    type Listener = ParsedListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        ToListener::to_listener(self.as_str())
    }
}

impl ToListener for &String {
    type Listener = ParsedListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        ToListener::to_listener(self.as_str())
    }
}

impl ToListener for &str {
    type Listener = ParsedListener;

    fn to_listener(self) -> io::Result<Self::Listener> {
        if let Ok(socket_addrs) = self.to_socket_addrs() {
            Ok(ParsedListener::Tcp(TcpListener::from_addrs(
                socket_addrs.collect(),
            )))
        } else if let Ok(url) = Url::parse(self) {
            ToListener::to_listener(url)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unable to parse listener from `{}`", self),
            ))
        }
    }
}

#[cfg(unix)]
impl ToListener for async_std::path::PathBuf {
    type Listener = UnixListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(UnixListener::from_path(self))
    }
}

#[cfg(unix)]
impl ToListener for std::path::PathBuf {
    type Listener = UnixListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(UnixListener::from_path(self))
    }
}

impl ToListener for async_std::net::TcpListener {
    type Listener = TcpListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(TcpListener::from_listener(self))
    }
}

impl ToListener for std::net::TcpListener {
    type Listener = TcpListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(TcpListener::from_listener(self))
    }
}

impl ToListener for (String, u16) {
    type Listener = TcpListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        ToListener::to_listener((self.0.as_str(), self.1))
    }
}

impl ToListener for (&String, u16) {
    type Listener = TcpListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        ToListener::to_listener((self.0.as_str(), self.1))
    }
}

impl ToListener for (&str, u16) {
    type Listener = TcpListener;

    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(TcpListener::from_addrs(self.to_socket_addrs()?.collect()))
    }
}

#[cfg(unix)]
impl ToListener for async_std::os::unix::net::UnixListener {
    type Listener = UnixListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(UnixListener::from_listener(self))
    }
}

#[cfg(unix)]
impl ToListener for std::os::unix::net::UnixListener {
    type Listener = UnixListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(UnixListener::from_listener(self))
    }
}

impl ToListener for TcpListener {
    type Listener = Self;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(self)
    }
}

#[cfg(unix)]
impl ToListener for UnixListener {
    type Listener = Self;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(self)
    }
}

impl ToListener for ConcurrentListener {
    type Listener = Self;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(self)
    }
}

impl ToListener for ParsedListener {
    type Listener = Self;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(self)
    }
}

impl ToListener for FailoverListener {
    type Listener = Self;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(self)
    }
}

impl ToListener for std::net::SocketAddr {
    type Listener = TcpListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        Ok(TcpListener::from_addrs(vec![self]))
    }
}

impl<L> ToListener for Vec<L>
where
    L: ToListener,
{
    type Listener = ConcurrentListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        let mut concurrent_listener = ConcurrentListener::new();
        for listener in self {
            concurrent_listener.add(listener)?;
        }
        Ok(concurrent_listener)
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    fn listen<L: ToListener>(listener: L) -> io::Result<L::Listener> {
        listener.to_listener()
    }

    #[test]
    fn url_to_tcp_listener() {
        let listener = listen(Url::parse("http://localhost:8000").unwrap()).unwrap();

        assert!(listener.to_string().contains("http://127.0.0.1:8000"));
        let listener = listen(Url::parse("tcp://localhost:8000").unwrap()).unwrap();
        assert!(listener.to_string().contains("http://127.0.0.1:8000"));

        let listener = listen(Url::parse("http://127.0.0.1").unwrap()).unwrap();
        assert_eq!(listener.to_string(), "http://127.0.0.1:80");
    }

    #[test]
    fn str_url_to_tcp_listener() {
        let listener = listen("tcp://localhost:8000").unwrap();
        assert!(listener.to_string().contains("http://127.0.0.1:8000"));

        let listener = listen("tcp://localhost:8000").unwrap();
        assert!(listener.to_string().contains("http://127.0.0.1:8000"));

        let listener = listen("tcp://127.0.0.1").unwrap();
        assert_eq!(listener.to_string(), "http://127.0.0.1:80");
    }

    #[cfg(unix)]
    mod unix {
        use super::*;

        #[test]
        fn str_url_to_unix_listener() {
            let listener = listen("http+unix:///var/run/tide/socket").unwrap();
            assert_eq!("http+unix:///var/run/tide/socket", listener.to_string());

            let listener = listen("http+unix://./socket").unwrap();
            assert_eq!("http+unix://./socket", listener.to_string());

            let listener = listen("http+unix://socket").unwrap();
            assert_eq!("http+unix://socket", listener.to_string());
        }

        #[test]
        fn colon_port_does_not_work() {
            let err = listen(":3000").unwrap_err().to_string();
            assert_eq!(err, "unable to parse listener from `:3000`");
        }
    }

    #[cfg(not(unix))]
    mod not_unix {
        use super::*;
        #[test]
        fn str_url_to_unix_listener() {
            let err = listen("http+unix:///var/run/tide/socket").unwrap_err();
            assert_eq!(
                err.to_string(),
                "Unix sockets not supported on this platform"
            );
        }

        #[test]
        fn colon_port_works() {
            let listener = listen(":3000").unwrap();
            assert!(listener.to_string().ends_with(":3000"));
            assert!(listener.to_string().starts_with("http://"));
        }
    }

    #[test]
    fn str_tls_parse_and_url() {
        let err = listen("tls://localhost:443").unwrap_err();
        assert_eq!(err.to_string(), "parsing TLS listeners not supported yet");

        let err = listen(Url::parse("https://localhost:443").unwrap()).unwrap_err();
        assert_eq!(err.to_string(), "parsing TLS listeners not supported yet");
    }

    #[test]
    fn str_unknown_scheme() {
        let err = listen("pigeon://localhost:443").unwrap_err();
        assert_eq!(err.to_string(), "unrecognized url scheme");

        let err = listen(Url::parse("pigeon:///localhost:443").unwrap()).unwrap_err();
        assert_eq!(err.to_string(), "unrecognized url scheme");
    }

    #[test]
    fn str_to_socket_addr() {
        let listener = listen("127.0.0.1:1312").unwrap();
        assert_eq!("http://127.0.0.1:1312", listener.to_string());

        let listener = listen("[::1]:1312").unwrap();
        assert_eq!("http://[::1]:1312", listener.to_string());

        let listener = listen("localhost:3000").unwrap();
        assert!(listener.to_string().contains(":3000"));
    }

    #[test]
    fn invalid_str_input() {
        let err = listen("hello world").unwrap_err();
        assert_eq!(
            err.to_string(),
            "unable to parse listener from `hello world`"
        );

        let err = listen("🌊").unwrap_err();
        assert_eq!(err.to_string(), "unable to parse listener from `🌊`");
    }

    #[test]
    fn to_listener_impls_compile() {
        listen("127.0.0.1:80").unwrap();
        listen(String::from("127.0.0.1:80")).unwrap();
        listen(&String::from("127.0.0.1:80")).unwrap();
        listen(("127.0.0.1", 80)).unwrap();
        listen((String::from("127.0.0.1"), 80)).unwrap();
        listen((&String::from("127.0.0.1"), 80)).unwrap();
    }
}
