- Start Date: 2019/06/02
- RFC PR: (leave this empty)
- Tracking Issue: (leave this empty)

# Summary
[summary]: #summary

This RFC tries to determine if extensible routing is needed in Tide, and if so
what approach should be used.

Note this is a continuation of [#271](https://github.com/rustasync/tide/issues/271).

# Motivation
[motivation]: #motivation

Discussions around PRs [#254](https://github.com/rustasync/tide/pull/254) (largely
on discord) and [#258](https://github.com/rustasync/tide/pull/258) addressed the question
of being able to replace or change the implementation of routing within Tide to
address different needs for different applications. Primarily these discussions
were about providing a simpler implementation for routing, for example purely
static routes for microservices, where the complexity that can be provided with
the existing implementation can be more than what is needed. Additionally, the
discussion raised the issue routing precedence that exists in the current
routing implementation used by tide. For example, if you have a URL
that contains ``/*path/:end`` it will match the ``*path`` first and therefore
the ``:end`` is not resolved independently and therefore parameters will be
consumed as part of the ``*path`` match and not provided to a ``:end``
parameter. This has been raised before as part of Issue
[#12](https://github.com/rustasync/tide/issues/12). If this issue is not resolved
directly with the default routing implementation then being able to have a
custom routing implementation allows for differing preferences in the routing
rules to be implemented.

# Stakeholders
[stakeholders]: #stakeholders

This affects everyone using Tide, but concerns two particular groups:
1. Users with specialized routing requirements - this is the group most likely to
   take advantage of customizing routing. The diverse nature of webservice
   use cases means that there is an advantage to future proofing the routing to
   allow for use cases outside of the existing frameworks we are used to.
2. New users of Tide - this is the group that the introduced complexity will
   affect most.

# Detailed Explanation
[detailed-explanation]: #detailed-explanation

This is a two part question - does Tide need extensible routing, and if so what
form should it take. This needs approaching from a high level, broad strokes level first
looking at the high level designs only to first answer the question, _does Tide
need extensible routing, or does having a good, understandable and configurable core
routing implementation meet the requirements_.

After first determining that the best approach for extension, if required, can be determined.

Looking back at the original [design goals](https://rustasync.github.io/team/2018/10/16/tide-routing.html)
(see the Digging Deeper section) the following goals were defined:

> - Make it very straightforward to understand how URLs map to code.
> - Make extraction and response serialization ergonomic
> - Avoid macros and code generation at the core; prefer simple, “plain Rust” APIs
> - Provide a clean mechanism for middleware and configuration

With the further clarification:

> For routing, to achieve the clarity goals, we follow these principles:
> - Separate out routing via a “table of contents” approach, making it easy to see the
    overall app structure.
> - No “fallback” in route matching; use match specificity. In particular, the order in
    which routes are added has no effect, and you cannot have two identical routes.
> - Drive endpoint selection solely by URL and HTTP method. Other aspects of a request
    can affect middleware and the behavior of the endpoint, but not which endpoint is
    used in the successful case. So for example, middleware can perform
    authentication and avoid invoking the endpoint on failure, but it does this by
    explicitly choosing a separate way of providing a response, rather than relying
    on “fallback” in the router.

The existing routing uses
[route-recognizer](https://github.com/conduit-rust/route-recognizer.rs)  which
doesn't appear to be maintained and has some implementation issues that need
addressing ([Route (metadata) ordering is incorrect.](https://github.com/conduit-rust/route-recognizer.rs/issues/20))
to provide a more robust routing implementation.  This is integrated into the core
of Tide, although WIP in PR #258 did extract this out into a separate module.

Discussion around this RFC has lead to a consensus that extensible routing for
Tide is a desirable outcome. This matches the original motivations of Tide _to
build a serious framework on top of these crates, ideally as a very minimal
layer_. Given it is hard to determine the exact use cases of everyone, but
desiring to provide a powerful framework this falls in line with providing means
to replace the routing implementation.

Therefore it makes sense to actually provide two means of extension:
1. Extract the existing implementation as traits to allow the replacing of the
core routing implementation as required.
1. Support routing as middleware through the exposing of the middleware traits
to allow extension and to be able to change the end point.


## Routing Traits
[routingtraits]: #routingtraits

For a custom routing implementation the main trait needed is the Router. This
provides the implementation for executing the routing taking in the possible
routes and then performing the routing based upon the request.

As a rough sketch this would look like:

```Rust
pub struct Selection<'a, State> {
  pub endpoint: &'a DynEndpoint<State>,
  pub params: Params,
}

pub trait Router<State> {
  fn add(&mut self, path: &str, method: http::Method, ep: impl Endpoint<State>);
  fn route(&self, path: &str, method: http::Method) -> Selection<'_, State>;
}

```

Note here I've continued the use of Params from the route-recognizer
implementation, but this should be extracted and made generically accessible.
This is essentially a ``BTreeMap<String, String>`` with some scaffolding to make
it more accessible.


## Middleware Support
[middlewaresupport]: #middlewaresupport

Routing in middleware is currently possible. The following is a simple example:

```Rust
#![feature(async_await)]
use tide::{cookies::ContextExt, middleware::{Middleware, Next}, Context, Response};
use futures::future::BoxFuture;
use http::StatusCode;

struct RoutingMiddleware
{
}

impl RoutingMiddleware {
    fn new() -> Self {
        RoutingMiddleware {
        }
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for RoutingMiddleware {
    fn handle<'a>(&'a self, cx: Context<State>, next: Next<'a, State>) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            let res = next.run(cx).await;
            let res = alt_handle().await;
            res
        })
    }
}

async fn alt_handle() -> Response {
    http::Response::builder()
        .status(StatusCode::OK)
        .body(String::from("Hello, alternative").into()).expect("Failed to create response")
}

fn main() {
    let mut app = tide::App::new();
    app.middleware(RoutingMiddleware::new());

    app.run("127.0.0.1:8000").unwrap();
}
```

The key here is that middleware has control over the response regardless of the
end point. Additionally, the routing rules do not require that any end point has
been defined before calling into the middleware. This means that you can change
the routing behavior in the middleware.

The limitation that exists is that the endpoint cannot be resolved by the
middleware and therefore be passed to subsequent middleware to perform their
operations. In the above example, if an existing routing was defined it would
therefore execute the existing end point before overwriting the result with the
alternative implementation.

Therefore, to make middleware routing meaningful the key is to provide the means
of changing the end point as part of the middleware implementation so that
routing can be inserting into other middleware structures without the potential
for having redundant calls the key is to provide configuring of the endpoint to
the middleware. This will also fulfil the requirements of the Internal redirects
issues ([#82](https://github.com/rustasync/tide/issues/82)).

The key is to allow the changing of the endpoint in the ``Next`` struct that is
provided to the middleware. Therefore an alternative function to forward the
request to another endpoint would make sense here:

```Rust
impl<'a, State: 'static> Next<'a, State> {
    /// Asynchronously execute the remaining middleware chain.
    pub fn run(mut self, cx: Context<State>) -> BoxFuture<'a, Response> {
      // ...
    }

    /// Asynchronously forward the request to the specified endpoint and execute
    /// the remaining middleware chain.
    pub fn forward(mut self, cx: Context<State>, &'a DynEndpoint<State>) -> BoxFuture<'a, Response> {
      // ...
    }
}
```

# Drawbacks
[drawbacks]: #drawbacks

There are three potential reasons why routing should remain as part of the
core of Tide:
1. Complexity of routing and internal redirects: Unlike other web server
   implementations the routing is the initial step and is driven through
   middleware until the end point is reached. Changing out the routing means a new
   mechanism for determining middleware integration, particularly if routing
   becomes a middleware implementation itself.

1. Tide routing is not checked at compile time, replacing the routing
   implementation is therefore not validated at compile time and changing the
   routing would mean having to potentially change the strings and paths
   significantly without guarantees beyond first startup. Arguably this is
   already a problem given that it isn't validated, but it is further complicated
   if the question of 'routing implementation' is added to the equation, depending
   on how routing is implemented in this case too.

1. Confusion for those learning Tide. This ties back to the lack of compile time
   validation, but more generally if you can change the routing implementation,
   and this isn't clearly visible in the documentation, tutorial or example then it
   is more likely to affect new users of Tide.

# Rationale and Alternatives
[alternatives]: #rationale-and-alternatives

It is worth looking at the possible alternative designs for routing
within Tide to get a feel for how this would look going forward with the
different options.

There are number of alternative designs to address routing:
1. Predefined configuration for routing

    This is sticking with the existing implementation. While this seems like the
simplest option it is not without work that needs to be done. There needs to
be further discussion about what constitutes the best implementation for
routing (e.g. a new RFC process). The existing routing has identifiable issues
with the predefined precedence of rules. [Route (metadata) ordering is incorrect.](https://github.com/conduit-rust/route-recognizer.rs/issues/20)
explains this in more detail with example code.

    This can be addressed in a number of ways (outside the scope of this RFC):

    1. Apply the PR [already supplied](https://github.com/conduit-rust/route-recognizer.rs/pull/21)
to route recognizer (either by getting upstream to do so or forking and creating
a new version).
    1. Replace route-recognizer with another implementation (e.g.
[path-tree](https://github.com/trek-rs/path-tree)) that meets the requirements

1. Configurable routing rules

    An alternative/extension of the previous is to allow configuring of the routing
rules, e.g. precedence rules can be defined through a builder implementation
when instantiating the App object. This would require more work and likely mean
forking and integrating the routing implementation within Tide itself to allow
the precedence rules to be configured.

1. Routing is static but internal redirects allow changing of routing rules.

    This is another extension of the previous options - whereby it puts the control
of configuring/extending the routing rules by middleware performing the changes
through internal redirects ([#82](https://github.com/rustasync/tide/issues/82)).

    This provides support for extending the Tide routing with custom routing rules
by putting the control within the middleware level. However it is likely to be
less performant than an implementation where this customization can be performed
at the initial parsing and evaluation of the routes.

1. Traits with a default implementation

    This involves scaffolding the implementation through Traits to allow for the
implementation to be changed through the particular implementation of the
required behavior through the traits. The instantiated implementation can then
be changed, or a default provided that performs the routing.

    This allows the routing to be changed out at compile time to meet the needs of
the individual projects. However it does mean have a clear and well defined set
of Traits that will capture all the required information for determining the
appropriate routing, and also path parsing.

1. Routing as middleware

    This moves the routing determination to the middleware implementation, passing
off the raw requests and allowing the middleware to determine which endpoint
should be called. This requires a default middleware implementation that
provides default routing rules, but allows replacing of the entire
routing implementation with more or less complexity as required.


# Unresolved Questions
[unresolved]: #unresolved-questions

What won't be resolved through this RFC. While these may come up in passing
because of determining the above questions it shouldn't be the focus of the
discussion:
- What the best default implementation is. I.e. this shouldn't become a
  discussion of the final precedence rules, or what the 'default' should look
  like.
- What routing implementation should be used.

# Notes
[notes]: #notes

Other open issues that relate to routing include:
- Internal redirects [#82](https://github.com/rustasync/tide/issues/82)
- URL generation [#24](https://github.com/rustasync/tide/issues/24)
- Permit routing without invoking actions [#155](https://github.com/rustasync/tide/issues/155)
- Implement OPTIONS http method [#51](https://github.com/rustasync/tide/issues/51)
- Allow the routing methods to open a static file [#63](https://github.com/rustasync/tide/issues/63)
