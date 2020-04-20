use async_std::io::Cursor;
use async_std::prelude::*;
use async_std::task;
use http_types::{headers, StatusCode};
use std::time::Duration;

use tide::Response;

const TEXT: &'static str = concat![
    "Eveniet delectus voluptatem in placeat modi. Qui nulla sunt aut non voluptas temporibus accusamus rem. Qui soluta nisi qui accusantium excepturi voluptatem. Ab rerum maiores neque ut expedita rem.",
    "Et neque praesentium eligendi quaerat consequatur asperiores dolorem. Pariatur tempore quidem animi consequuntur voluptatem quos. Porro quo ipsa quae suscipit. Doloribus est qui facilis ratione. Delectus ex perspiciatis ab alias et quisquam non est.",
    "Id dolorum distinctio distinctio quos est facilis commodi velit. Ex repudiandae aliquam eos voluptatum et. Provident qui molestiae molestiae nostrum voluptatum aperiam ut. Quis repellendus quidem mollitia aut recusandae laboriosam.",
    "Corrupti cupiditate maxime voluptatibus totam neque facilis. Iure deleniti id incidunt in sunt suscipit ea. Hic ullam qui doloribus tempora voluptas. Unde id debitis architecto beatae dolores autem et omnis. Impedit accusamus laudantium voluptatem ducimus.",
    "Eos maxime hic aliquid accusantium. Et voluptas sit accusamus modi natus. Et voluptatem sequi ea et provident voluptatum minus voluptas. Culpa aliquam architecto consequatur animi.",
];

#[async_std::test]
async fn chunked_large() -> Result<(), http_types::Error> {
    let server = task::spawn(async {
        let mut app = tide::new();
        app.at("/").get(|mut _req: tide::Request<()>| async move {
            let body = Cursor::new(TEXT.to_owned());
            let res = Response::new(StatusCode::Ok)
                .body(body)
                .set_header(headers::CONTENT_TYPE, "text/plain; charset=utf-8");
            Ok(res)
        });
        app.listen("localhost:8080").await?;
        Result::<(), http_types::Error>::Ok(())
    });

    let client = task::spawn(async {
        task::sleep(Duration::from_millis(100)).await;
        let mut res = surf::get("http://localhost:8080").await?;
        assert_eq!(res.status(), 200);
        assert_eq!(
            res.header(&"transfer-encoding".parse().unwrap()),
            Some(&vec![http_types::headers::HeaderValue::from_ascii(
                b"chunked"
            )
            .unwrap()])
        );
        assert_eq!(res.header(&"content-length".parse().unwrap()), None);
        let string = res.body_string().await?;
        assert_eq!(string, TEXT.to_string());
        Ok(())
    });

    server.race(client).await
}
