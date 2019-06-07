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
address different needs for different applications. The existing routing within
Tide has issues that need addressing ([#12](https://github.com/rustasync/tide/issues/12)) and how these issues are resolved tie
into the question of extensible routing.

# Stakeholders
[stakeholders]: #stakeholders

This affects everyone using Tide, but concerns two particular groups:
1. Users with specialized routing requirements - this is the group most likely to
   take advantage of customizing routing.
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

Other open issues that relate to routing include:
- Internal redirects [#82](https://github.com/rustasync/tide/issues/82)
- URL generation [#24](https://github.com/rustasync/tide/issues/24)
- Permit routing without invoking actions [#155](https://github.com/rustasync/tide/issues/155)
- Implement OPTIONS http method [#51](https://github.com/rustasync/tide/issues/51)
- Allow the routing methods to open a static file [#63](https://github.com/rustasync/tide/issues/63)

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
simplest option it is not without working that needs to be done. There needs to
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

The aim of this RFC is to determine the 'if' and the broad 'how' of extensible
routing in Tide. To that end the unresolved questions are:

- Assuming a good default routing implementation, does Tide need extensible
  routing?
- If routing can be configured (e.g. precedence rules), does Tide need
  extensible routing?
- If Tide needs extensible routing what should that look like at the high level?
  E.g. Traits, Middleware, Configuration only

What won't be resolved through this RFC. While these may come up in passing
because of determining the above questions it shouldn't be the focus of the
discussion:
- What the best default implementation is. I.e. this shouldn't become a
  discussion of the final precedence rules, or what the 'default' should look
  like.
- What routing implementation should be used.
