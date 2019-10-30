use http::status::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tide::{error::ResultExt, response, App, Context, EndpointResult};

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

#[allow(unused_mut)] // Workaround clippy bug
async fn new_message(mut cx: Context<Database>) -> EndpointResult<String> {
    let msg = cx.body_json().await.client_err()?;
    Ok(cx.state().insert(msg).to_string())
}

#[allow(unused_mut)] // Workaround clippy bug
async fn set_message(mut cx: Context<Database>) -> EndpointResult<()> {
    let msg = cx.body_json().await.client_err()?;
    let id = cx.param("id").client_err()?;

    if cx.state().set(id, msg) {
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND)?
    }
}

async fn get_message(cx: Context<Database>) -> EndpointResult {
    let id = cx.param("id").client_err()?;
    if let Some(msg) = cx.state().get(id) {
        Ok(response::json(msg))
    } else {
        Err(StatusCode::NOT_FOUND)?
    }
}

fn main() {
    let mut app = App::with_state(Database::default());
    app.at("/message").post(new_message);
    app.at("/message/:id").get(get_message).post(set_message);
    app.serve("127.0.0.1:8000").unwrap();
}
