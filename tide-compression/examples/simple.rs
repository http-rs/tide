use tide::{App, Context};
use tide_compression::{Compression, Decompression, Encoding};

// Returns a portion of the lorem ipsum text.
async fn lorem_ipsum(_cx: Context<()>) -> String {
    String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit.")
}

// Echoes the request body in bytes.
async fn echo_bytes(mut cx: Context<()>) -> Vec<u8> {
    cx.body_bytes().await.unwrap()
}

fn main() {
    let mut app = App::new();
    app.at("/").get(lorem_ipsum);
    app.at("/echo").post(echo_bytes);
    app.middleware(Compression::with_default(Encoding::Brotli));
    app.middleware(Decompression::new());
    app.run("127.0.0.1:8000").unwrap();
}
