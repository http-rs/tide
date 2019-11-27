use http::status::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tide::{Request, Result, ResultExt, Server};

#[derive(Default)]
struct Database {
    contents: Mutex<Vec<Message>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    author: Option<String>,
    contents: String,
}

impl Database {
    fn insert(&self, msg: Message) -> usize {
        let mut table = self.contents.lock().unwrap();
        table.push(msg);
        table.len() - 1
    }

    fn get(&self, id: usize) -> Option<Message> {
        self.contents.lock().unwrap().get(id).cloned()
    }

    fn set(&self, id: usize, msg: Message) -> bool {
        let mut table = self.contents.lock().unwrap();

        if let Some(old_msg) = table.get_mut(id) {
            *old_msg = msg;
            true
        } else {
            false
        }
    }
}

async fn new_message(mut cx: Request<Database>) -> Result<String> {
    let msg = cx.body_json().await.client_err()?;
    Ok(cx.state().insert(msg).to_string())
}

async fn set_message(mut cx: Request<Database>) -> Result<()> {
    let msg = cx.body_json().await.client_err()?;
    let id = cx.param("id").client_err()?;

    if cx.state().set(id, msg) {
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND)?
    }
}

async fn get_message(cx: Request<Database>) -> Result {
    let id = cx.param("id").client_err()?;
    if let Some(msg) = cx.state().get(id) {
        Ok(cx.body_json())
    } else {
        Err(StatusCode::NOT_FOUND)?
    }
}

fn main() {
    let mut app = Server::with_state(Database::default());
    app.at("/message").post(new_message);
    app.at("/message/:id").get(get_message).post(set_message);
    app.run("127.0.0.1:8000").unwrap();
}
