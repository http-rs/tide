- Start Date: 2019-04-26
- RFC PR: (leave this empty)
- Tracking Issue: (leave this empty)

# Summary
[summary]: #summary

We propose to rename the `AppData` generic argument to `State`, and expose two constructors on `App`
to create new Tide applications.

# Motivation
[motivation]: #motivation

With most introductory examples, passing the `AppData` parameter around isn't necessary. That's why
most examples we author use `App::new(())`.

However this can be confusing for people new to Tide ("why is the use of that parameter?"), and even
stranger for people new to Rust ("what does `(())` mean?"). So it would be useful to have a way of
constructing new Tide applications without needing to start off by explaining what `AppData` does.

Another thing worth thinking about here is that the `AppData` argument is rather verbose. It's also
not necessarily accurate: I'd argue that a database connection pool, or other stateful structs
aren't quite _data_. Calling them _state_ feels more accurate, which also happens to be the
terminology [Actix uses](https://actix.rs/docs/databases/).

# Stakeholders
[stakeholders]: #stakeholders

This affects everyone using Tide. But most generally it's geared towards improving the onboarding
experience, and first impressions for people of all skill levels. Not just for people new to Tide,
but people interested in doing web-like things in Rust in general.

# Detailed Explanation
[detailed-explanation]: #detailed-explanation

I propose we introduce two constructors for `App`:

- `App::new()` creates a new application.
- `App::with_state(state)` creates a new application with state.

If people "just want a Tide app", the `new` method should feel intuitive. But if they want to
introduce some state, the `with_state` method will be there. This should also create a clearer
relationship between the constructors, and the generic parameters we have in Tide.

In addition
[`Context::app_data`](https://docs.rs/tide/0.1.1/tide/struct.Context.html#method.app_data) should be
renamed to `Context::state`.

__no state__
```rust
fn main() -> Result<(), failure::Error> {
    let mut app = tide::App::new();
    app.at("/").get(|_| async move { "Hello, world!" });
    app.serve("127.0.0.1:8000")?;
}
```

__with state__
```rust
#[derive(Default)]
struct State {
  /* db connection goes here */
}

fn main() -> Result<(), failure::Error> {
    let mut app = tide::App::with_state(State::default());
    app.at("/").get(|_| async move { "Hello, world!" });
    app.serve("127.0.0.1:8000")?;
}
```

# Drawbacks
[drawbacks]: #drawbacks

This is a breaking change. But because it's still early days for Tide, I think right now is the
right time to propose changes like these.

# Rationale and Alternatives
[alternatives]: #rationale-and-alternatives

In https://github.com/rustasync/tide/pull/189 I initially proposed implementing `Default` for `App`,
but it quickly became clear that making more fundamental changes to Tide's constructors would
provide a better experience. In particular clearing up the terminology around `AppData` by calling
it `State` made all other parts fall in place naturally.

# Unresolved Questions
[unresolved]: #unresolved-questions

None.
