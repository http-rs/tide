# Changelog

All notable changes to tide will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://book.async.rs/overview/stability-guarantees.html).

## [Unreleased]

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

[Unreleased]: https://github.com/http-rs/tide/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/http-rs/tide/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/http-rs/tide/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/http-rs/tide/compare/v0.0.5...v0.1.0
[0.0.5]: https://github.com/http-rs/tide/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/http-rs/tide/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/http-rs/tide/compare/v0.0.1...v0.0.3
[0.0.1]: https://github.com/http-rs/tide/compare/v0.0.1
