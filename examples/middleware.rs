use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tide::http::mime;
use tide::utils::{After, Before};
use tide::{Middleware, Next, Request, Response, Result, StatusCode};

#[derive(Debug)]
struct User {
    name: String,
}

#[derive(Clone, Default, Debug)]
struct UserDatabase;
impl UserDatabase {
    async fn find_user(&self) -> Option<User> {
        Some(User {
            name: "nori".into(),
        })
    }
}

// This is an example of a function middleware that uses the
// application state. Because it depends on a specific request state,
// it would likely be closely tied to a specific application
fn user_loader<'a>(
    mut request: Request<UserDatabase>,
    next: Next<'a, UserDatabase>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
    Box::pin(async {
        if let Some(user) = request.state().find_user().await {
            tide::log::trace!("user loaded", {user: user.name});
            request.set_ext(user);
            Ok(next.run(request).await)
        // this middleware only needs to run before the endpoint, so
        // it just passes through the result of Next
        } else {
            // do not run endpoints, we could not find a user
            Ok(Response::new(StatusCode::Unauthorized))
        }
    })
}

// This is an example of middleware that keeps its own state and could
// be provided as a third party crate
#[derive(Default)]
struct RequestCounterMiddleware {
    requests_counted: Arc<AtomicUsize>,
}

impl RequestCounterMiddleware {
    fn new(start: usize) -> Self {
        Self {
            requests_counted: Arc::new(AtomicUsize::new(start)),
        }
    }
}

struct RequestCount(usize);

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for RequestCounterMiddleware {
    async fn handle(&self, mut req: Request<State>, next: Next<'_, State>) -> Result {
        let count = self.requests_counted.fetch_add(1, Ordering::Relaxed);
        tide::log::trace!("request counter", { count: count });
        req.set_ext(RequestCount(count));

        let mut res = next.run(req).await;

        res.insert_header("request-number", count.to_string());
        Ok(res)
    }
}

const NOT_FOUND_HTML_PAGE: &str = "<html><body>
  <h1>uh oh, we couldn't find that document</h1>
  <p>
    probably, this would be served from the file system or
    included with `include_bytes!`
  </p>
</body></html>";

const INTERNAL_SERVER_ERROR_HTML_PAGE: &str = "<html><body>
  <h1>whoops! it's not you, it's us</h1>
  <p>
    we're very sorry, but something seems to have gone wrong on our end
  </p>
</body></html>";

#[async_std::main]
async fn main() -> Result<()> {
    tide::log::start();
    let mut app = tide::with_state(UserDatabase::default());

    app.with(After(|response: Response| async move {
        let response = match response.status() {
            StatusCode::NotFound => Response::builder(404)
                .content_type(mime::HTML)
                .body(NOT_FOUND_HTML_PAGE)
                .build(),

            StatusCode::InternalServerError => Response::builder(500)
                .content_type(mime::HTML)
                .body(INTERNAL_SERVER_ERROR_HTML_PAGE)
                .build(),

            _ => response,
        };

        Ok(response)
    }));

    app.with(user_loader);
    app.with(RequestCounterMiddleware::new(0));
    app.with(Before(|mut request: Request<UserDatabase>| async move {
        request.set_ext(std::time::Instant::now());
        request
    }));

    app.at("/").get(|req: Request<_>| async move {
        let count: &RequestCount = req.ext().unwrap();
        let user: &User = req.ext().unwrap();

        Ok(format!(
            "Hello {}, this was request number {}!",
            user.name, count.0
        ))
    });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
