#![feature(async_await, futures_api)]

use http::status::StatusCode;
use juniper::graphql_object;
use std::sync::{atomic, Arc};
use tide::{body, App, AppData, Response};

#[derive(Clone, Default)]
struct Context(Arc<atomic::AtomicIsize>);

impl juniper::Context for Context {}

struct Query;

graphql_object!(Query: Context |&self| {
    field accumulator(&executor) -> i32 as "Current value of the accumulator" {
        executor.context().0.load(atomic::Ordering::Relaxed) as i32
    }
});

struct Mutation;

graphql_object!(Mutation: Context |&self| {
    field add(&executor, by: i32) -> i32 as "Add given value to the accumulator." {
        executor.context().0.fetch_add(by as isize, atomic::Ordering::Relaxed) as i32 + by
    }
});

type Schema = juniper::RootNode<'static, Query, Mutation>;

async fn handle_graphql(
    ctx: AppData<Context>,
    query: body::Json<juniper::http::GraphQLRequest>,
) -> Result<Response, StatusCode> {
    let request = query.0;
    let response = request.execute(&Schema::new(Query, Mutation), &ctx);

    let body_vec = serde_json::to_vec(&response)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    http::Response::builder()
        .status(if response.is_ok() { StatusCode::OK } else { StatusCode::BAD_REQUEST })
        .header("Content-Type", "application/json")
        .body(body::Body::from(body_vec))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn main() {
    let mut app = App::new(Context::default());

    app.at("/graphql").post(handle_graphql);

    app.serve("127.0.0.1:7878");
}
