#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(test, deny(warnings))]
#![allow(unused_variables)]
#![feature(
    futures_api,
    async_await,
    await_macro,
    pin,
    arbitrary_self_types,
    existential_type
)]

mod app;
pub mod body;
mod config;
mod endpoint;
mod extract;
pub mod head;
pub mod middleware;
mod request;
mod response;
mod router;
mod typemap;

pub use crate::{
    app::{App, AppData},
    endpoint::Endpoint,
    extract::Extract,
    middleware::Middleware,
    request::{Compute, Computed, Request},
    response::{IntoResponse, Response},
    router::{Resource, Router},
};
pub use path_table::RouteMatch;
