use tide::*;

#[async_std::test]
async fn test_status() {
    let mut resp = Response::new(StatusCode::NotFound).body_string("foo".to_string());
    assert_eq!(resp.status(), StatusCode::NotFound);

    let foo = resp.take_body().into_string().await.unwrap();
    assert_eq!(foo.as_bytes(), b"foo");
}

#[async_std::test]
async fn byte_vec_content_type() {
    use async_std::io::Cursor;
    use http_types::headers::{HeaderName, HeaderValue};
    use std::str::FromStr;

    let mut resp = Response::new(StatusCode::Ok).body(Cursor::new("foo"));

    let mut header_values: Vec<HeaderValue> = Vec::new();
    let content_type = HeaderName::from_str("Content-Type").unwrap();
    if let Some(x) = resp.remove_header(&content_type) {
        header_values = x;
    }
    assert_eq!(header_values[0], "application/octet-stream");

    let foo = resp.take_body().into_string().await.unwrap();
    assert_eq!(foo.as_bytes(), b"foo");
}

#[async_std::test]
async fn string_content_type() {
    use http_types::headers::{HeaderName, HeaderValue};
    use std::str::FromStr;

    let mut resp = Response::new(StatusCode::Ok).body_string("foo".to_string());

    let mut header_values: Vec<HeaderValue> = Vec::new();
    let content_type = HeaderName::from_str("Content-Type").unwrap();
    if let Some(x) = resp.remove_header(&content_type) {
        header_values = x;
    }
    assert_eq!(header_values[0], "text/plain; charset=utf-8");

    let foo = resp.take_body().into_string().await.unwrap();
    assert_eq!(foo.as_bytes(), b"foo");
}

#[async_std::test]
async fn json_content_type() {
    use http_types::Method;
    use std::collections::BTreeMap;

    let mut app = tide::new();
    app.at("/json_content_type").get(|_| async move {
        let mut map = BTreeMap::new();
        map.insert(Some("a"), 2);
        map.insert(Some("b"), 4);
        map.insert(None, 6);
        println!("{:?}", serde_json::to_vec(&map));

        let resp = Response::new(StatusCode::Ok).body_json(&map)?;
        Ok(resp)
    });

    let req = http::Request::new(
        Method::Get,
        "http://localhost/json_content_type".parse().unwrap(),
    );
    let mut resp: http::Response = app.respond(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::InternalServerError);

    let body = resp.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"");

    let mut map = BTreeMap::new();
    map.insert("a", 2);
    map.insert("b", 4);
    map.insert("c", 6);

    let mut resp = Response::new(StatusCode::Ok).body_json(&map).unwrap();
    assert_eq!(resp.status(), StatusCode::Ok);

    let body = resp.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), br##"{"a":2,"b":4,"c":6}"##);
}
