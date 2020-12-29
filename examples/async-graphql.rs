use tide::{http::mime, Body, Response, StatusCode};

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};

/// query example for async-graphql
///
/// # simple query for users
///
/// ```
/// {
///   allUsers {
///     id
///     firstName
///   }
/// }
/// ```

struct User {
    id: Option<u16>,
    first_name: String,
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

struct Query;

#[async_graphql::Object]
impl Query {
    async fn all_users(&self) -> Vec<User> {
        let user1 = User { id: Some(12), first_name: "Alice".to_string() };
        let user2 = User { id: Some(22), first_name: "Jack".to_string() };
        let user3 = User { id: Some(32), first_name: "Tom".to_string() };

        vec![user1, user2, user3]
    }
}

#[derive(Clone)]
struct AppState {
    schema: Schema<Query, EmptyMutation, EmptySubscription>,
}

#[async_std::main]
async fn main() ->  Result<(), std::io::Error> {
    let schema = Schema::new(
        Query, 
        EmptyMutation, 
        EmptySubscription
    );

    tide::log::start();

    let mut app = tide::new();

    app.at("/graphql")
        .post(async_graphql_tide::endpoint(schema));
    app.at("/graphiql").get(|_| async move {
        let mut resp = Response::new(StatusCode::Ok);
        resp.set_body(Body::from_string(playground_source(
            GraphQLPlaygroundConfig::new("/graphql"),
        )));
        resp.set_content_type(mime::HTML);
        Ok(resp)
    });

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
