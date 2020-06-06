mod test_utils;

use tide::http::{self, headers, mime, StatusCode};
use tide::Response;

#[async_std::test]
async fn test_status() {
    let mut resp = Response::new(StatusCode::NotFound);
    resp.set_body("foo");
    assert_eq!(resp.status(), StatusCode::NotFound);
    let foo = resp.take_body().into_string().await.unwrap();
    assert_eq!(foo.as_bytes(), b"foo");
}

#[async_std::test]
async fn byte_vec_content_type() {
    use async_std::io::Cursor;
    use tide::Body;

    let mut resp = Response::new(StatusCode::Ok);
    resp.set_body(Body::from_reader(Cursor::new("foo"), None));

    assert_eq!(resp[headers::CONTENT_TYPE], mime::BYTE_STREAM.to_string());
    let foo = resp.take_body().into_bytes().await.unwrap();
    assert_eq!(foo, b"foo");
}

#[async_std::test]
async fn string_content_type() {
    let mut resp = Response::new(StatusCode::Ok);
    resp.set_body("foo");

    assert_eq!(resp[headers::CONTENT_TYPE], mime::PLAIN.to_string());
    let foo = resp.take_body().into_string().await.unwrap();
    assert_eq!(foo, "foo");
}

#[async_std::test]
async fn json_content_type() {
    use std::collections::BTreeMap;
    use tide::http::{Method, Url};
    use tide::Body;

    let mut app = tide::new();
    app.at("/json_content_type").get(|_| async move {
        let mut map = BTreeMap::new();
        map.insert(Some("a"), 2);
        map.insert(Some("b"), 4);
        map.insert(None, 6);
        let mut resp = Response::new(StatusCode::Ok);
        resp.set_body(Body::from_json(&map)?);
        Ok(resp)
    });
    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/json_content_type").unwrap(),
    );
    let mut resp: http::Response = app.respond(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::InternalServerError);
    let body = resp.take_body().into_bytes().await.unwrap();
    assert_eq!(body, b"");

    let mut resp = async move {
        let mut map = BTreeMap::new();
        map.insert("a", 2);
        map.insert("b", 4);
        map.insert("c", 6);
        let mut r = Response::new(StatusCode::Ok);
        r.set_body(Body::from_json(&map).unwrap());
        r
    }
    .await;
    assert_eq!(resp.status(), StatusCode::Ok);
    let body = resp.take_body().into_bytes().await.unwrap();
    assert_eq!(body, br##"{"a":2,"b":4,"c":6}"##);
}
