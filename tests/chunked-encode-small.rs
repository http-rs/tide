mod test_utils;
use async_std::io::Cursor;
use async_std::prelude::*;
use async_std::task;
use std::time::Duration;

use tide::Body;

const TEXT: &str = concat![
    "Eveniet delectus voluptatem in placeat modi. Qui nulla sunt aut non voluptas temporibus accusamus rem. Qui soluta nisi qui accusantium excepturi voluptatem. Ab rerum maiores neque ut expedita rem.",
    "Et neque praesentium eligendi quaerat consequatur asperiores dolorem. Pariatur tempore quidem animi consequuntur voluptatem quos. Porro quo ipsa quae suscipit. Doloribus est qui facilis ratione. Delectus ex perspiciatis ab alias et quisquam non est.",
    "Id dolorum distinctio distinctio quos est facilis commodi velit. Ex repudiandae aliquam eos voluptatum et. Provident qui molestiae molestiae nostrum voluptatum aperiam ut. Quis repellendus quidem mollitia aut recusandae laboriosam.",
    "Corrupti cupiditate maxime voluptatibus totam neque facilis. Iure deleniti id incidunt in sunt suscipit ea. Hic ullam qui doloribus tempora voluptas. Unde id debitis architecto beatae dolores autem et omnis. Impedit accusamus laudantium voluptatem ducimus.",
    "Eos maxime hic aliquid accusantium. Et voluptas sit accusamus modi natus. Et voluptatem sequi ea et provident voluptatum minus voluptas. Culpa aliquam architecto consequatur animi.",
];

#[async_std::test]
async fn chunked_large() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.at("/")
            .get(|_, _| async { Ok(Body::from_reader(Cursor::new(TEXT), None)) });
        app.listen(("localhost", port)).await?;
        Result::<(), http_types::Error>::Ok(())
    });

    let client = task::spawn(async move {
        task::sleep(Duration::from_millis(100)).await;
        let mut res = surf::get(format!("http://localhost:{}", port))
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.header("transfer-encoding").unwrap(), "chunked");
        assert!(res.header("content-length").is_none());
        let string = res.body_string().await.unwrap();
        assert_eq!(string, TEXT.to_string());
        Ok(())
    });

    server.race(client).await
}
