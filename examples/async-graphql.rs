use async_std::sync::{Arc, RwLock};

use tide::{Request, Response, Body, StatusCode};
use tide::http::mime;

use async_graphql::{EmptySubscription, Schema, Context};
use async_graphql::http::{receive_json, playground_source, GraphQLPlaygroundConfig};

/// query example for async-graphql
///
/// # simple query: allUsers1
///
/// ```
/// {
///   allUsers1 {
///     id
///     firstName
///   }
/// }
/// ```
/// 
/// # simple mutation: addUser
///
/// ```
/// mutation {
///     addUser(user:{firstName:"John"}) {
///       id
///       firstName
///     }
///   }
/// ```
///
/// # then, simple query: allUsers2
///
/// ```
/// {
///   allUsers2 {
///     id
///     firstName
///   }
/// }
/// ```

#[derive(Clone)]
struct User {
    id: Option<u16>,
    first_name: String
}

#[async_graphql::Object]
impl User {
    async fn id(&self) -> i32 {
        self.id.unwrap_or(0) as i32
    }

    async fn first_name(&self) -> &str {
        &self.first_name
    }
}

#[derive(async_graphql::InputObject)]
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

#[derive(Default)]
pub struct Users(Arc<RwLock<Vec<User>>>);

pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    /// Get all Users: macro vec!
    async fn all_users1(&self) -> Vec<User> {
        let user1 = User { id: Some(12), first_name: "Alice".to_string() };
        let user2 = User { id: Some(22), first_name: "Jack".to_string() };
        let user3 = User { id: Some(32), first_name: "Tom".to_string() };

        vec![user1, user2, user3]
    }

    /// Get all Users: method addUser(user:NewUser)
    async fn all_users2(&self, ctx: &Context<'_>) -> Vec<User> {
        let users = ctx.data_unchecked::<Users>().0.read().await;

        users.iter().cloned().collect()
    }
}

pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    /// Add new user
    async fn add_user(&self, ctx: &Context<'_>, user: NewUser) -> User {
        let mut users = ctx.data_unchecked::<Users>().0.write().await;
        let mut user = user.into_internal();

        user.id = Some((users.len() + 1) as u16);
        users.push(user.clone());

        user
    }
}

#[derive(Clone)]
pub struct State(Schema<QueryRoot, MutationRoot, EmptySubscription>);

async fn handle_graphql(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let schema = req.state().0.clone();
    let gql_resp = schema.execute(receive_json(req).await?).await;
    
    let mut resp = Response::new(StatusCode::Ok);
    resp.set_body(Body::from_json(&gql_resp)?);

    Ok(resp)
}

async fn handle_playground(_: Request<State>) -> tide::Result<impl Into<Response>> {
    let mut resp = Response::new(StatusCode::Ok);
    resp.set_body(playground_source(GraphQLPlaygroundConfig::new("/graphql")));
    resp.set_content_type(mime::HTML);

    Ok(resp)
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();

    // let mut schema = Schema::new(QueryRoot, MutationRoot, EmptySubscription);
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(Users::default())
        .finish();
        
    let mut app = tide::with_state(State(schema));

    app.at("/graphql").post(handle_graphql);
    app.at("/graphiql").get(handle_playground);
    
    app.listen("127.0.0.1:8080").await?;
    
    Ok(())
}
