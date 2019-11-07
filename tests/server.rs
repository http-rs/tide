use async_std::task;
use async_std::future;
use std::time::Duration;

#[test]
fn hello_world() -> Result<(), surf::Exception> {
    task::block_on(async {
        let server = task::spawn(async {
            let mut app = tide::new();
            app.at("/").get(|_| async {"hello world"});
            app.listen("localhost:8080").await?;
            Result::<(), surf::Exception>::Ok(())
        });

        let client = task::spawn(async {
            task::sleep(Duration::from_millis(100)).await;
            let string = surf::get("localhost:8080").recv_string().await?;
            assert_eq!(string, "hello world");
            Ok(())
        });

        future::select!(server, client).await
    })
}
