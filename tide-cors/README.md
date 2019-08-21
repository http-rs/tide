# tide-cors

This crate provides cors-related middleware for Tide.

## Examples

Examples are in the `/examples` folder of this crate.

```rust,no_run
use http::header::HeaderValue;
use tide::middleware::CorsMiddleware;

fn main() {
    let mut app = tide::App::new();

    app.middleware(
        CorsMiddleware::new()
            .allow_origin(HeaderValue::from_static("*"))
            .allow_methods(HeaderValue::from_static("GET, POST, OPTIONS")),
    );

    app.at("/").get(|_| async move { "Hello, world!" });

    app.run("127.0.0.1:8000").unwrap();
}
```

__Simple Example__

You can test the simple example by running `cargo run --example cors` while in this crate's directory, and then running this script in the browser console:

```console
$ fetch("http://127.0.0.1:8000")
```

You will probably get a browser alert when running without cors middleware.
