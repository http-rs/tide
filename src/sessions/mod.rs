//! # Tide session support
//!
//! This document provides a high-level overview of tide's approach to
//! sessions. For implementation and examples, please refer to
//! [SessionMiddleware](crate::sessions::SessionMiddleware)
//!
//! Sessions allows tide to securely attach data to a browser session
//! allowing for retrieval and modification of this data within tide
//! on subsequent visits. Session data is generally only retained for
//! the duration of a browser session.
//!
//! Tide's session implementation provides guest sessions by default,
//! meaning that all web requests to a session-enabled tide host will
//! have a cookie attached, whether or not there is anything stored in
//! that client's session yet.
//!
//! ## Stores
//!
//! Although tide provides two bundled session stores, it is highly
//! recommended that tide applications use an
//! external-datastore-backed session storage. For a list of currently
//! available session stores, see [the documentation for
//! async-session](https://github.com/http-rs/async-session).
//!
//! ## Security
//!
//! Although each session store may have different security
//! implications, the general approach of tide's session system is as
//! follows: On each request, tide checks the cookie configurable as
//! `cookie_name` on the middleware.
//!
//! ### If no cookie is found:
//!
//! A cryptographically random cookie value is generated. A cookie is
//! set on the outbound response and signed with an HKDF key derived
//! from the `secret` provided on creation of the SessionMiddleware.
//! The configurable session store uses a SHA256 digest of the cookie
//! value and stores the session along with a potential expiry.
//!
//! ### If a cookie is found:
//!
//! The hkdf derived signing key is used to verify the cookie value's
//! signature. If it verifies, it is then passed to the session store
//! to retrieve a Session. For most session stores, this will involve
//! taking a SHA256 digest of the cookie value and retrieving a
//! serialized Session from an external datastore based on that
//! digest.
//!
//! ### Expiry
//!
//! In addition to setting an expiry on the session cookie, tide
//! sessions include the same expiry in their serialization format. If
//! an adversary were able to tamper with the expiry of a cookie, tide
//! sessions would still check the expiry on the contained session
//! before using it
//!
//! ### If anything goes wrong with the above process
//!
//! If there are any failures in the above session retrieval process,
//! a new empty session is generated for the request, which proceeds
//! through the application as normal.
//!
//! ## Stale/expired session cleanup
//!
//! Any session store other than the cookie store will accumulate
//! stale sessions.  Although the tide session middleware ensures that
//! they will not be used as valid sessions, For most session stores,
//! it is the tide application's responsibility to call cleanup on the
//! session store if it requires it
//!

pub use middleware::SessionMiddleware;

mod middleware;

pub use async_session::{CookieStore, MemoryStore, Session, SessionStore};
