use async_std::task;
use juniper::RootNode;
use std::sync::RwLock;
use tide::{Body, Redirect, Request, Response, Server, StatusCode};

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
    fn to_internal(self) -> User {
        User {
            id: None,
            first_name: self.first_name.to_owned(),
        }
    }
}

pub struct State {
    users: RwLock<Vec<User>>,
}
impl juniper::Context for State {}

pub struct QueryRoot;

#[juniper::object(Context=State)]
impl QueryRoot {
    #[graphql(description = "Get all Users")]
    fn users(context: &State) -> Vec<User> {
        let users = context.users.read().unwrap();
        users.iter().map(|u| u.clone()).collect()
    }
}

pub struct MutationRoot;

#[juniper::object(Context=State)]
impl MutationRoot {
    #[graphql(description = "Add new user")]
    fn add_user(context: &State, user: NewUser) -> User {
        let mut users = context.users.write().unwrap();
        let mut user = user.to_internal();
        user.id = Some((users.len() + 1) as u16);
        users.push(user.clone());
        user
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;
fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}

async fn handle_graphql(mut req: Request<State>) -> tide::Result {
    let query: juniper::http::GraphQLRequest = req
        .body_json()
        .await
        .expect("be able to deserialize the graphql request");

    let schema = create_schema(); // probably worth making the schema a singleton using lazy_static library
    let response = query.execute(&schema, req.state());
    let status = if response.is_ok() {
        StatusCode::Ok
    } else {
        StatusCode::BadRequest
    };

    let mut res = Response::new(status);
    res.set_body(Body::from_json(&response)?);
    Ok(res)
}

async fn handle_graphiql(_: Request<State>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(juniper::http::graphiql::graphiql_source("/graphql"));
    res.set_content_type(tide::http::mime::HTML);
    Ok(res)
}

fn main() -> std::io::Result<()> {
    task::block_on(async {
        let mut app = Server::with_state(State {
            users: RwLock::new(Vec::new()),
        });
        app.at("/").get(Redirect::permanent("/graphiql"));
        app.at("/graphql").post(handle_graphql);
        app.at("/graphiql").get(handle_graphiql);
        app.listen("0.0.0.0:8080").await?;
        Ok(())
    })
}
