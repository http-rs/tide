// This example uses Juniper to process GraphQL requests. If you're not familiar with Juniper, take
// a look at [the Juniper book].
//
// [the Juniper book]: https://graphql-rust.github.io/
use http::status::StatusCode;
use juniper::graphql_object;
use std::sync::{atomic, Arc};
use tide::{error::ResultExt, response, App, Context, EndpointResult};

// First, we define `Data` that holds accumulator state. This is accessible as App data in
// Tide, and as executor context in Juniper.
#[derive(Clone, Default)]
struct Data(Arc<atomic::AtomicIsize>);

impl juniper::Context for Data {}

// We define `Query` unit struct here. GraphQL queries will refer to this struct. The struct itself
// doesn't have any associated data (and there's no need to do so), but instead it exposes the
// accumulator state from the context.
struct Query;

graphql_object!(Query: Data |&self| {
    // GraphQL integers are signed and 32 bits long.
    field accumulator(&executor) -> i32 as "Current value of the accumulator" {
        executor.context().0.load(atomic::Ordering::Relaxed) as i32
    }
});

// Here is `Mutation` unit struct. GraphQL mutations will refer to this struct. This is similar to
// `Query`, but it provides the way to "mutate" the accumulator state.
struct Mutation;

graphql_object!(Mutation: Data |&self| {
    field add(&executor, by: i32) -> i32 as "Add given value to the accumulator." {
        executor.context().0.fetch_add(by as isize, atomic::Ordering::Relaxed) as i32 + by
    }
});

// Adding `Query` and `Mutation` together we get `Schema`, which describes, well, the whole GraphQL
// schema.
type Schema = juniper::RootNode<'static, Query, Mutation>;

// Finally, we'll bridge between Tide and Juniper. `GraphQLRequest` from Juniper implements
// `Deserialize`, so we use `Json` extractor to deserialize the request body.
async fn handle_graphql(mut cx: Context<Data>) -> EndpointResult {
    let query: juniper::http::GraphQLRequest = cx.body_json().await.client_err()?;
    let schema = Schema::new(Query, Mutation);
    let response = query.execute(&schema, cx.state());
    let status = if response.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    let mut resp = response::json(response);
    *resp.status_mut() = status;
    Ok(resp)
}

pub fn main() {
    let mut app = App::with_state(Data::default());
    app.at("/graphql").post(handle_graphql);
    app.serve("127.0.0.1:8000").unwrap();
}
