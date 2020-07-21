use std::sync::{Arc, RwLock};

use juniper::{http::graphiql, http::GraphQLRequest, RootNode};
use lazy_static::lazy_static;
use tide::{http::mime, Body, Redirect, Request, Response, Server, StatusCode};

#[derive(Clone)]
struct User {
    id: Option<u16>,
    first_name: String,
}

#[juniper::object]
#[graphql(description = "A user")]
impl User {
    #[graphql(description = "A user id")]
    fn id(&self) -> i32 {
        self.id.unwrap_or(0) as i32
    }

    #[graphql(description = "A user first_name")]
    fn first_name(&self) -> &str {
        &self.first_name
    }
}

#[derive(juniper::GraphQLInputObject)]
struct NewUser {
    first_name: String,
}

impl NewUser {
    fn into_internal(self) -> User {
        User {
            id: None,
            first_name: self.first_name,
        }
    }
}

#[derive(Clone)]
pub struct State {
    users: Arc<RwLock<Vec<User>>>,
}
impl juniper::Context for State {}

pub struct QueryRoot;

#[juniper::object(Context=State)]
impl QueryRoot {
    #[graphql(description = "Get all Users")]
    fn users(context: &State) -> Vec<User> {
        let users = context.users.read().unwrap();
        users.iter().cloned().collect()
    }
}

pub struct MutationRoot;

#[juniper::object(Context=State)]
impl MutationRoot {
    #[graphql(description = "Add new user")]
    fn add_user(context: &State, user: NewUser) -> User {
        let mut users = context.users.write().unwrap();
        let mut user = user.into_internal();
        user.id = Some((users.len() + 1) as u16);
        users.push(user.clone());
        user
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;
lazy_static! {
    static ref SCHEMA: Schema = Schema::new(QueryRoot {}, MutationRoot {});
}

async fn handle_graphql(mut request: Request<State>) -> tide::Result {
    let query: GraphQLRequest = request.body_json().await?;
    let response = query.execute(&SCHEMA, request.state());
    let status = if response.is_ok() {
        StatusCode::Ok
    } else {
        StatusCode::BadRequest
    };

    Ok(Response::builder(status)
        .body(Body::from_json(&response)?)
        .build())
}

async fn handle_graphiql(_: Request<State>) -> tide::Result<impl Into<Response>> {
    Ok(Response::builder(200)
        .body(graphiql::graphiql_source("/graphql"))
        .content_type(mime::HTML))
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let mut app = Server::with_state(State {
        users: Arc::new(RwLock::new(Vec::new())),
    });
    app.at("/").get(Redirect::permanent("/graphiql"));
    app.at("/graphql").post(handle_graphql);
    app.at("/graphiql").get(handle_graphiql);
    app.listen("0.0.0.0:8080").await?;
    Ok(())
}
