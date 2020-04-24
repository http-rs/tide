<h1 align="center">Tide</h1>
<div align="center">
 <strong>
   Serve the web
 </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/tide">
    <img src="https://img.shields.io/crates/v/tide.svg?style=flat-square"
    alt="Crates.io version" />
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
    <a href="https://github.com/http-rs/tide/blob/master/.github/CONTRIBUTING.md">
      Contributing
    </a>
    <span> | </span>
    <a href="https://discord.gg/x2gKzst">
      Chat
    </a>
  </h3>
</div>

A modular web framework built around async/await. It's actively being developed
and **not ready for production yet**.

## Getting started

Add two dependencies to your project's `Cargo.toml` file: `tide` itself, and `async-std` with the feature `attributes` enabled:
```toml
# Example, use the version numbers you need
tide = "0.7.0"
async-std = { version = "1.5.0", features = ["attributes"] }
```

## Examples

**Hello World**

```rust
#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();
    app.at("/").get(|_| async { Ok("Hello, world!") });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
```

To try [the included examples](https://github.com/http-rs/tide/tree/master/examples), check out this repository and run
```sh
$ cargo run --example # shows a list of available examples
$ cargo run --example hello
```

## Resources

Read about the design here:

- [Rising Tide: building a modular web framework in the open](https://rustasync.github.io/team/2018/09/11/tide.html)
- [Routing and extraction in Tide: a first sketch](https://rustasync.github.io/team/2018/10/16/tide-routing.html)
- [Middleware in Tide](https://rustasync.github.io/team/2018/11/07/tide-middleware.html)
- [Tide's evolving middleware approach](https://rustasync.github.io/team/2018/11/27/tide-middleware-evolution.html)
- [Tide, the present and future of](https://blog.yoshuawuyts.com/tide/)

## Contributing

Want to join us? Check out our [The "Contributing" section of the
guide][contributing] and take a look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

#### Conduct

The Tide project adheres to the [Contributor Covenant Code of
Conduct](https://github.com/http-rs/tide/blob/master/.github/CODE_OF_CONDUCT.md).
This describes the minimum behavior expected from all contributors.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[releases]: https://github.com/http-rs/tide/releases
[contributing]: https://github.com/http-rs/tide/blob/master/.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/http-rs/tide/labels/good%20first%20issue
[help-wanted]: https://github.com/http-rs/tide/labels/help%20wanted
