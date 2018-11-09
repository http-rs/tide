#![feature(async_await, futures_api)]

async fn echo_string(msg: String) -> String {
  println!("String: {}", msg);
  format!("{}", msg)
}

async fn echo_vec(msg: Vec<u8>) -> String {
  println!("Vec<u8>: {:?}", msg);

  String::from_utf8(msg).unwrap()
}

fn main() {
  let mut app = tide::App::new(());
  app.at("/echo/string").post(echo_string);
  app.at("/echo/vec").post(echo_vec);
  app.serve("127.0.0.1:8000");
}
