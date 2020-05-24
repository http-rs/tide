mod test_utils;
use async_std::prelude::*;
use async_std::sync;
use async_std::task;
use std::time::Duration;

use tide::{JobContext, Request};

#[test]
fn jobs() -> Result<(), http_types::Error> {
    #[derive(Default)]
    struct Counter {
        count: sync::Mutex<i32>,
    }

    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
            let mut app = tide::with_state(Counter::default());
            app.at("/").get(|req: Request<Counter>| async move {
                Ok(req.state().count.lock().await.to_string())
            });

            app.spawn(|ctx: JobContext<Counter>| async move {
                *ctx.state().count.lock().await += 1;
            });

            app.listen(("localhost", port)).await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(200)).await;
            let string = surf::get(format!("http://localhost:{}", port))
                .recv_string()
                .await
                .unwrap();
            assert_eq!(string, "1".to_string());
            Ok(())
        });

        server.race(client).await
    })
}
