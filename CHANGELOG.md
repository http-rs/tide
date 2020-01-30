# Changelog

All notable changes to tide will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://book.async.rs/overview/stability-guarantees.html).

## [Unreleased]

## [0.6.0] - 2020-01-30

[API Documentation](https://docs.rs/tide/0.6.0/tide)

This patch introduces a new cookies API, based on the excellent
[cookie](https://docs.rs/cookie/) crate. Working with cookies is a staple for
any web server, and Tide's new API now makes this entirely declarative.

Additionally we've added back CORS support. This makes it possible for
possible to configure the single-origin policy of browsers, which is an
incredibly valuable resource.

And finally nesting services with Tide has become even easier. Building on
the APIs in 0.5.0, the manual song-and-dance required to nest APIs is no
longer required, and services can now be nested as-is through the
`Route::nest` API.

### Examples

#### Cookies

```rust
use cookie::Cookie;
use tide::Response;

let mut app = tide::new();

app.at("/").get(|req| async move {
    println!("cat snack: {:?}", req.cookie("snack"));
    Response::new(200)
});
app.at("/set").get(|req| async move {
    let mut res = Response::new(200);
    res.set_cookie(Cookie::new("snack", "tuna"));
    res
});
app.listen("127.0.0.1:8080").await?;
```

#### CORS

Make GET, POST, and OPTIONS endpoints on this server accessible from any web
page.

```rust
use http::header::HeaderValue;
use tide::middleware::{Cors, Origin};

let rules = Cors::new()
    .allow_methods(HeaderValue::from_static("GET, POST, OPTIONS"))
    .allow_origin(Origin::from("*"))
    .allow_credentials(false);

let mut app = tide::new();
app.middleware(rules);
app.at("/").post(|_| async { Response::new(200) });
app.listen("localhost:8080").await?;
```

#### Nesting

Nest the inner serve inside the outer service, exposing `GET /nori/cat`.

```rust
let mut inner = tide::new();
inner.at("/nori").get(|_| async { Response::new(200) });

let mut outer = tide::new();
outer.at("/cat").nest(inner);

outer.listen("localhost:8080").await?;
```

### Added

- Added `Route::all` to match all HTTP methods on a route ([#379](https://github.com/http-rs/tide/pull/379))
- Added `Route::nest` to nest instances of `tide::Server` on sub-routes ([#379](https://github.com/http-rs/tide/pull/379))
- Added a new `cors` submodule containing CORS control middleware ([#373](https://github.com/http-rs/tide/pull/373))
- Added `Request::cookie` to get a cookie sent by the client ([#380](https://github.com/http-rs/tide/pull/380/files))
- Added `Response::set_cookie` to instruct the client to set a cookie ([#380](https://github.com/http-rs/tide/pull/380/files))
- Added `Response::remove_cookie` to instruct the client to unset a cookie ([#380](https://github.com/http-rs/tide/pull/380/files))

### Changed

- Changed the behavior of optional params in `Request.query` to be more intuitive ([384](https://github.com/http-rs/tide/pull/384))
- Improved the debugging experience of query deserialization errors ([384](https://github.com/http-rs/tide/pull/384))
- Updated the GraphQL example to use the latest version of Juniper ([#372](https://github.com/http-rs/tide/pull/372))
- Tide no longer prints to stdout when started ([387](https://github.com/http-rs/tide/pull/387))

### Fixed

- Fixed an incorrect MIME type definition on `Response::body` ([378](https://github.com/http-rs/tide/pull/378))

## [0.5.1] - 2019-12-20

[API Documentation](https://docs.rs/tide/0.5.1/tide)

This fixes a rendering issue on docs.rs.

## Fixes

- Fix a rendering issue on docs.rs [(#376)](https://github.com/http-rs/tide/pull/376)

## [0.5.0] - 2019-12-20

[API Documentation](https://docs.rs/tide/0.5.0/tide)

This release introduces the ability to nest applications, add logging
middleware, and improves our documentation.

Nesting applications is a useful technique that can be used to create several
sub-applications. This allows creating clear points of isolation in applications
that can be used completely independently of the main application. But can be
recombined into a single binary if required.

Being able to nest applications is also a necessary first step to re-introduce
per-route middleware, which we'll do in subsequent patches.

## Examples

```rust
let mut inner = tide::new();
inner.at("/").get(|_| async { "root" });
inner.at("/foo").get(|_| async { "foo" });
inner.at("/bar").get(|_| async { "bar" });

let mut outer = tide::new();
outer
    .at("/nested")
    .strip_prefix() // catch /nested and /nested/*
    .get(inner.into_http_service()); // the prefix /nested will be stripped here
```

## Added

- Added `Route::strip_prefix` [(#364)](https://github.com/http-rs/tide/pull/364)
- Added the ability `Service`s to be nested [(#364)](https://github.com/http-rs/tide/pull/364)
- Added `middleware::RequestLogger` [(#367)](https://github.com/http-rs/tide/pull/367)

## Changed

- Updated and improved the documentation [(#363)](https://github.com/http-rs/tide/pull/363)

## [0.4.0] - 2019-11-26

This release is a further polishing of Tide's APIs, and works towards
significantly improving Tide's user experience. The biggest question left
unanswered after this patch is how we want to do error handling, but aside from
that the end-user API should be pretty close to where we want it to be.

The biggest changes in this patch is endpoints now take `Request` instead of
`Context`. The new `Request` and `Response` types are no longer type aliases but
concrete types, making them substantially easier to use. This also means that
we've been able to fold in all the `Ext` methods we were exposing, enabling
methods such as `let values: Schema = req.body_json()?;` to deserialize an
incoming JSON body through a `Serde` schema. This should make it significantly
easier to write APIs with Tide out of the box.

## Example

Create a "hello world" app:
```rust
#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();
    app.at("/").get(|_| async move { "Hello, world!" });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
```

Redirect from `/nori` to `/chashu`:

```rust
#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();
    app.at("/chashu").get(|_| async move { "meow" });
    app.at("/nori").get(tide::redirect("/chashu"));
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
```

### Added

- Added `logger::RequestLogger` based on `log` (replaces `logger:RootLogger`).
- Added `Request` with inherent methods (replaces `Context`).
- Added `Server` (replaces `App`).
- Added `Response` (replacing a type alias of the same name).
- Added a `prelude` submodule, holding all public traits.
- Added a `new` free function, a shorthand for `Server::new`.
- Added a `with_state` free function, a shorthand for `Server::with_state`.
- Added `Result` type alias (replaces `EndpointResult`).
- Added a `redirect` free function to redirect from one endpoint to another.

### Changed

- Resolved an `#[allow(unused_mut)]` workaround.
- Renamed `ExtractForms` to `ContextExt`.
- `Response` is now a newly defined type.

### Removed

- Removed `logger::RootLogger` (replaced by `logger:RequestLogger`).
- Removed internal use of the `box_async` macro.
- Removed `Context` (replaced by `Request`).
- Removed the `Response` type alias (replaced by a new `Response` struct).
- Removed `App` (replaced by `Server`).
- Temporarily disabled the multipart family of APIs, improving compilation
  speed by ~30%.
- Removed `EndpointResult` (replaced by `Result`).

## [0.3.0] - 2019-10-31

This is the first release in almost 6 months; introducing a snapshot of where we
were right before splitting up the crate. This release is mostly similar to
`0.2.0`, but sets us up to start rewinding prior work on top.

### Added

- Added "unstable" feature flag.
- Added example for serving static files.
- Added keywords and categories to Cargo.toml.
- Implemented `Default` for `App`.
- Added `App::with_state` constructor method.
- Added `Context::state` (replacing `Request::app_data`)
- Added examples to the documentation root.
- Added a section about stability guarantees to the documentation root.

### Changed

- Fixed multipart uploads.
- Fixed some doc tests.
- Rename `cookies::CookiesExt` to `cookies::ContextExt`.
- Rename `querystring::ExtractQuery` to `querystring::ContextExt`.
- Switched CI provider from Travis to GitHub actions.
- Updated README.
- Updated all dependencies.
- Replaced `AppData` with `State`.

### Removed

- Removed the RFCs subdirectory.
- Removed an extra incoming license requirement.
- Removed outdated version logs.
- Removed `rustfmt.toml`.
- Removed `Request::app_data` (replaced with `Context::state`).

## [0.2.0] - 2019-05-03

Log not kept.

## [0.1.1] - 2019-04-18

Log not kept.

## [0.1.0] - 2019-04-15

Log not kept.

## [0.0.5] - 2019-02-26

Log not kept.

## [0.0.4] - 2019-02-04

Log not kept.

## [0.0.3] - 2019-01-31

Log not kept.

## [0.0.1] - 2019-01-18

Log not kept.

[Unreleased]: https://github.com/http-rs/tide/compare/0.5.1...HEAD
[0.5.1]: https://github.com/http-rs/tide/compare/0.5.0...0.5.1
[0.5.0]: https://github.com/http-rs/tide/compare/0.4.0...0.5.0
[0.4.0]: https://github.com/http-rs/tide/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/http-rs/tide/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/http-rs/tide/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/http-rs/tide/compare/0.0.5...0.1.0
[0.0.5]: https://github.com/http-rs/tide/compare/0.0.4...0.0.5
[0.0.4]: https://github.com/http-rs/tide/compare/0.0.3...0.0.4
[0.0.3]: https://github.com/http-rs/tide/compare/0.0.1...0.0.3
[0.0.1]: https://github.com/http-rs/tide/compare/0.0.1
