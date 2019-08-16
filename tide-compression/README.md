# tide-compression

This crate provides compression-related middleware for Tide.

## Examples

Examples are in the `/examples` folder of this crate.

__Simple Example__

You can test the simple example by running `cargo run --example simple` while in this crate's directory, and then running any variety of the following commands:

```console
$ curl -v http://127.0.0.1:8000/
$ curl -v -H 'Accept-Encoding: br' http://127.0.0.1:8000/
$ echo 'hello there' | gzip | curl -v --compressed -H 'Content-Encoding: gzip' http://127.0.0.1:8000/echo --data-binary @-
$ echo 'general kenobi' | brotli | curl -v --compressed -H 'Content-Encoding: br' http://127.0.0.1:8000/echo --data-binary @-
```
