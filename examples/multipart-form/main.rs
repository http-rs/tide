#![feature(async_await, futures_api)]

#[macro_use]
extern crate serde_derive;

use http::status::StatusCode;
use std::io::Read;
use tide::{body, App};

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    key1: Option<String>,
    key2: Option<String>,
    file: Option<String>,
}

async fn upload_file(
    multipart_form: body::MultipartForm,
) -> Result<body::Json<Message>, StatusCode> {
    // https://stackoverflow.com/questions/43424982/how-to-parse-multipart-forms-using-abonander-multipart-with-rocket
    let mut message = Message {
        key1: None,
        key2: None,
        file: None,
    };

    let mut multipart = multipart_form.0;
    multipart
        .foreach_entry(|mut entry| match entry.headers.name.as_str() {
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

    Ok(body::Json(message))
}

fn main() {
    let mut app = App::new(());

    app.at("/upload_file").post(upload_file);

    app.serve("127.0.0.1:7878");
}

// Test with:
// curl -X POST http://localhost:7878/upload_file -H 'content-type: multipart/form-data' -F file=@examples/multipart-form/test.txt
// curl -X POST http://localhost:7878/upload_file -H 'content-type: multipart/form-data' -F key1=v1, -F key2=v2
