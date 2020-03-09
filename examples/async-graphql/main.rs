mod starwars;

use async_graphql::http::{graphiql_source, playground_source, GQLRequest};
use async_graphql::{GQLEmptyMutation, Schema};
use mime;
use tide::{self, Request, Response};

type StarWarsSchema = Schema<starwars::QueryRoot, GQLEmptyMutation>;

async fn index(mut request: Request<StarWarsSchema>) -> Response {
    let gql_request: GQLRequest = request.body_json().await.unwrap();
    let schema = request.state();
    let gql_response = gql_request.execute(schema).await;
    Response::new(200).body_json(&gql_response).unwrap()
}

async fn gql_playground(_request: Request<StarWarsSchema>) -> Response {
    Response::new(200)
        .body_string(playground_source("/"))
        .set_mime(mime::TEXT_HTML_UTF_8)
}
async fn gql_graphiql(_request: Request<StarWarsSchema>) -> Response {
    Response::new(200)
        .body_string(graphiql_source("/"))
        .set_mime(mime::TEXT_HTML_UTF_8)
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let mut app = tide::with_state(
        Schema::new(starwars::QueryRoot, GQLEmptyMutation).data(starwars::StarWars::new()),
    );
    app.at("/").post(index);
    app.at("/").get(gql_playground);
    app.at("/graphiql").get(gql_graphiql);
    app.listen("0.0.0.0:8000").await
}
