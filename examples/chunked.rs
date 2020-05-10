use async_std::fs;
use async_std::io::BufReader;
use async_std::task;
use tide::{Response, StatusCode};

fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let mut app = tide::new();
        app.at("/").get(|_| async move {
            let file = fs::File::open(file!()).await.unwrap();
            let res = Response::new(StatusCode::Ok).body(BufReader::new(file));
            Ok(res)
        });
        app.listen("127.0.0.1:8080").await
    })
}
