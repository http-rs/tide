// This example uses Juniper to process GraphQL requests. If you're not familiar with Juniper, take
// a look at [the Juniper book].
//
// [the Juniper book]: https://graphql-rust.github.io/

#![feature(async_await, futures_api)]

use http::status::StatusCode;
use juniper::graphql_object;
use std::sync::{atomic, Arc};
use tide::{body, App, AppData, Response};

// First, we define `Context` that holds accumulator state. This is accessible as App data in
// Tide, and as executor context in Juniper.
#[derive(Clone, Default)]
struct Context(Arc<atomic::AtomicIsize>);

impl juniper::Context for Context {}

// We define `Query` unit struct here. GraphQL queries will refer to this struct. The struct itself
// doesn't have any associated data (and there's no need to do so), but instead it exposes the
// accumulator state from the context.
struct Query;

graphql_object!(Query: Context |&self| {
    // GraphQL integers are signed and 32 bits long.
    field accumulator(&executor) -> i32 as "Current value of the accumulator" {
        executor.context().0.load(atomic::Ordering::Relaxed) as i32
    }
});

// Here is `Mutation` unit struct. GraphQL mutations will refer to this struct. This is similar to
// `Query`, but it provides the way to "mutate" the accumulator state.
struct Mutation;

graphql_object!(Mutation: Context |&self| {
    field add(&executor, by: i32) -> i32 as "Add given value to the accumulator." {
        executor.context().0.fetch_add(by as isize, atomic::Ordering::Relaxed) as i32 + by
    }
});

// Adding `Query` and `Mutation` together we get `Schema`, which describes, well, the whole GraphQL
// schema.
type Schema = juniper::RootNode<'static, Query, Mutation>;

// Finally, we'll bridge between Tide and Juniper. `GraphQLRequest` from Juniper implements
// `Deserialize`, so we use `Json` extractor to deserialize the request body.
async fn handle_graphql(
    ctx: AppData<Context>,
    query: body::Json<juniper::http::GraphQLRequest>,
) -> Result<Response, StatusCode> {
    let request = query.0;
    let response = request.execute(&Schema::new(Query, Mutation), &ctx);

    // `response` has the lifetime of `request`, so we can't use `IntoResponse` directly.
    let body_vec = serde_json::to_vec(&response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    http::Response::builder()
        .status(if response.is_ok() {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        })
        .header("Content-Type", "application/json")
        .body(body::Body::from(body_vec))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn main() {
    let mut app = App::new(Context::default());

    app.at("/graphql").post(handle_graphql);

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address);
}
