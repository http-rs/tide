use async_std::task;
use serde::{Deserialize, Serialize};
use std::io::Read;
use tide::{Request, Response, StatusCode};

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    key1: Option<String>,
    key2: Option<String>,
    file: Option<String>,
}

async fn upload_file(mut cx: Request<()>) -> tide::Result {
    // https://stackoverflow.com/questions/43424982/how-to-parse-multipart-forms-using-abonander-multipart-with-rocket
    let mut message = Message {
        key1: None,
        key2: None,
        file: None,
    };

    cx.body_multipart::<Message>()
        .await?
        .foreach_entry(|mut entry| match &*entry.headers.name {
            "file" => {
                let mut vec = Vec::new();
                entry.data.read_to_end(&mut vec).expect("can't read");
                message.file = String::from_utf8(vec).ok();
                println!("key file got");
            }

            "key1" => {
                let mut vec = Vec::new();
                entry.data.read_to_end(&mut vec).expect("can't read");
                message.key1 = String::from_utf8(vec).ok();
                println!("key1 got");
            }

            "key2" => {
                let mut vec = Vec::new();
                entry.data.read_to_end(&mut vec).expect("can't read");
                message.key2 = String::from_utf8(vec).ok();
                println!("key2 got");
            }

            _ => {
                // as multipart has a bug https://github.com/abonander/multipart/issues/114
                // we manually do read_to_end here
                let mut _vec = Vec::new();
                entry.data.read_to_end(&mut _vec).expect("can't read");
                println!("key neglected");
            }
        })
        .expect("Unable to iterate multipart?");

    Response::new(StatusCode::Ok)
        .body_json(&message)
        .map_err(|e| http_types::Error::new(StatusCode::InternalServerError, e))
}

fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let mut app = tide::new();
        app.at("/upload_file").post(upload_file);
        app.listen("127.0.0.1:8000").await?;
        Ok(())
    })
}

// Test with:
// curl -X POST http://localhost:8000/upload_file -H 'content-type: multipart/form-data' -F file=@examples/multipart-form/test.txt
// curl -X POST http://localhost:8000/upload_file -H 'content-type: multipart/form-data' -F key1=v1, -F key2=v2
