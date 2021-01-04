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
    <a href="https://github.com/http-rs/tide/blob/main/.github/CONTRIBUTING.md">
      Contributing
    </a>
    <span> | </span>
    <a href="https://discord.gg/x2gKzst">
      Chat
    </a>
  </h3>
</div>

Tide is a minimal and pragmatic Rust web application framework built for
rapid development. It comes with a robust set of features that make building
async web applications and APIs easier and more fun.

## Getting started

In order to build a web app in Rust you need an HTTP server, and an async
runtime. After running `cargo init` add the following lines to your
`Cargo.toml` file:

```toml
# Example, use the version numbers you need
tide = "0.15.0"
async-std = { version = "1.8.0", features = ["attributes"] }
serde = { version = "1.0", features = ["derive"] }
```

## Examples

Create an HTTP server that receives a JSON body, validates it, and responds
with a confirmation message.

```rust
use tide::Request;
use tide::prelude::*;

#[derive(Debug, Deserialize)]
struct Animal {
    name: String,
    legs: u8,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/orders/shoes").post(order_shoes);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn order_shoes(mut req: Request<()>) -> tide::Result {
    let Animal { name, legs } = req.body_json().await?;
    Ok(format!("Hello, {}! I've put in an order for {} shoes", name, legs).into())
}
```

```sh
$ curl localhost:8080/orders/shoes -d '{ "name": "Chashu", "legs": 4 }'
Hello, Chashu! I've put in an order for 4 shoes

$ curl localhost:8080/orders/shoes -d '{ "name": "Mary Millipede", "legs": 750 }'
number too large to fit in target type
```

See more examples in the [examples](https://github.com/http-rs/tide/tree/main/examples) directory.

## Tide's design:
- [Rising Tide: building a modular web framework in the open](https://rustasync.github.io/team/2018/09/11/tide.html)
- [Routing and extraction in Tide: a first sketch](https://rustasync.github.io/team/2018/10/16/tide-routing.html)
- [Middleware in Tide](https://rustasync.github.io/team/2018/11/07/tide-middleware.html)
- [Tide's evolving middleware approach](https://rustasync.github.io/team/2018/11/27/tide-middleware-evolution.html)
- [Tide, the present and future of](https://blog.yoshuawuyts.com/tide/)
- [Tide channels](https://blog.yoshuawuyts.com/tide-channels/)

## Community Resources
<sub>To add a link to this list, [edit the markdown
file](https://github.com/http-rs/tide/edit/main/README.md) and
submit a pull request (github login required)</sub><br/><sup>Listing here
does not constitute an endorsement or recommendation from the tide
team. Use at your own risk.</sup>

### Listeners
* [tide-rustls](https://github.com/http-rs/tide-rustls) tls for tide based on async-tls/rustls

### Template engines
* [tide-tera](https://github.com/jbr/tide-tera)
* [tide-handlebars](https://github.com/No9/tide-handlebars)
* [askama](https://github.com/djc/askama) (includes support for tide)

### Routers
* [tide-fluent-routes](https://github.com/mendelt/tide-fluent-routes)

### Auth
* [tide-http-auth](https://github.com/chrisdickinson/tide-http-auth)

### Testing
* [tide-testing](https://github.com/jbr/tide-testing)

### Middleware
* [tide-compress](https://github.com/Fishrock123/tide-compress)
* [tide-sqlx](https://github.com/eaze/tide-sqlx) - _SQLx pooled connections & transactions_
* [tide-trace](https://github.com/no9/tide-trace)
* [tide-tracing](https://github.com/ethanboxx/tide-tracing)
* [opentelemetry-tide](https://github.com/asaaki/opentelemetry-tide)
* [driftwood](https://github.com/jbr/driftwood) http logging middleware
* [tide-compressed-sse](https://github.com/Yarn/tide_compressed_sse)
* [tide-websockets](https://github.com/http-rs/tide-websockets)

### Session Stores
* [async-redis-session](https://github.com/jbr/async-redis-session)
* [async-sqlx-session](https://github.com/jbr/async-sqlx-session) (sqlite and postgres currently)
* [async-mongodb-session](https://github.com/yoshuawuyts/async-mongodb-session/)

### Example applications
* [dot dot vote](https://github.com/rtyler/dotdotvote/)
* [tide-example](https://github.com/jbr/tide-example) (sqlx + askama)
* [playground-tide-mongodb](https://github.com/yoshuawuyts/playground-tide-mongodb)
* [tide-morth-example](https://github.com/No9/tide-morth-example/)
* [tide-async-graphql-mongodb-boilerplate](https://github.com/zzy/tide-async-graphql-mongodb)

## Contributing
Want to join us? Check out our [The "Contributing" section of the
guide][contributing] and take a look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

#### Conduct

The Tide project adheres to the [Contributor Covenant Code of
Conduct](https://github.com/http-rs/tide/blob/main/.github/CODE_OF_CONDUCT.md).
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
[contributing]: https://github.com/http-rs/tide/blob/main/.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/http-rs/tide/labels/good%20first%20issue
[help-wanted]: https://github.com/http-rs/tide/labels/help%20wanted
