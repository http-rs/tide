# Changelog

All notable changes to tide will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://book.async.rs/overview/stability-guarantees.html).

## [Unreleased]

### Added

- Added `logger::RequestLogger` based on `log` (replaces `logger:RootLogger`)

### Changed

- Resolved an `#[allow(unused_mut)]` workaround.
- Renamed `ExtractForms` to `ContextExt`.

### Removed

- Removed `logger::RootLogger` (replaced by `logger:RequestLogger`)
- Removed internal use of the `box_async` macro

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
