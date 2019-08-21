<h1 align="center">Tide</h1>
<div align="center">
 <strong>
   Empowering everyone to build HTTP Services.
 </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/tide">
    <img src="https://img.shields.io/crates/v/tide.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Build Status -->
  <a href="https://travis-ci.org/rustasync/tide">
    <img src="https://img.shields.io/travis/rustasync/tide.svg?style=flat-square"
      alt="Build Status" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/tide">
    <img src="https://img.shields.io/crates/d/tide.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/tide">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://docs.rs/tide">
      API Docs
    </a>
    <span> | </span>
    <a href="https://github.com/rustasync/tide/blob/master/.github/CONTRIBUTING.md">
      Contributing
    </a>
    <span> | </span>
    <a href="https://discordapp.com/channels/442252698964721669/474974025454452766">
      Chat
    </a>
  </h3>
</div>

<div align="center">
  <sub>Built with 🌊 by <a href="https://github.com/rustasync">The Rust Async Ecosystem WG</a>
</div>

## About

A modular web framework built around async/await. It's actively being developed by the Rust Async
Ecosystem WG, and **not ready for production use yet**.

## Examples

**Hello World**

```rust,no_run
fn main() -> Result<(), std::io::Error> {
    let mut app = tide::App::new();
    app.at("/").get(|_| async move { "Hello, world!" });
    Ok(app.run("127.0.0.1:8000")?)
}
```

**More Examples**

- [Hello World](https://github.com/rustasync/tide/blob/master/examples/hello.rs)
- [Messages](https://github.com/rustasync/tide/blob/master/examples/messages.rs)
- [Body Types](https://github.com/rustasync/tide/blob/master/examples/body_types.rs)
- [Multipart Form](https://github.com/rustasync/tide/blob/master/examples/multipart_form/mod.rs)
- [Catch All](https://github.com/rustasync/tide/blob/master/examples/catch_all.rs)
- [Cookies](https://github.com/rustasync/tide/blob/master/examples/cookies.rs)
- [Default Headers](https://github.com/rustasync/tide/blob/master/examples/default_headers.rs)
- [GraphQL](https://github.com/rustasync/tide/blob/master/examples/graphql.rs)
- [Staticfile](https://github.com/rustasync/tide/blob/master/examples/staticfile.rs)

## Resources

Read about the design here:

- [Rising Tide: building a modular web framework in the open](https://rustasync.github.io/team/2018/09/11/tide.html)
- [Routing and extraction in Tide: a first sketch](https://rustasync.github.io/team/2018/10/16/tide-routing.html)
- [Middleware in Tide](https://rustasync.github.io/team/2018/11/07/tide-middleware.html)
- [Tide's evolving middleware approach](https://rustasync.github.io/team/2018/11/27/tide-middleware-evolution.html)

### Supported Rust Versions

Tide is built against the latest Rust nightly releases and as such, due to it's use of `std` futures,
it has the following specific breakpoints that align with std future API changes:

| Tide        | Rust                    |
| ----------- | ----------------------- |
| &le; v0.1.0 | &le; nightly-2019-04-07 |
| &ge; v0.1.1 | &ge; nightly-2019-04-08 |

_**Note:** Since these are due to changes in `std`, projects with dependencies that use conflicting versions of `std::futures` will not build successfully._

## Contributing

Want to join us? Check out our [The "Contributing" section of the
guide][contributing] and take a look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

#### Conduct

The Tide project adheres to the [Contributor Covenant Code of
Conduct](https://github.com/rustasync/tide/blob/master/.github/CODE_OF_CONDUCT.md). This
describes the minimum behavior expected from all contributors.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[releases]: https://github.com/rustasync/tide/releases
[contributing]: https://github.com/rustasync/tide/blob/master/.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/rustasync/tide/labels/good%20first%20issue
[help-wanted]: https://github.com/rustasync/tide/labels/help%20wanted
