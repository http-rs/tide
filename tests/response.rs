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
    use http_types::headers::HeaderName;
    use std::str::FromStr;

    let mut resp = Response::new(StatusCode::Ok).body(Cursor::new("foo"));
    let header_values = resp
        .header(&HeaderName::from_str("Content-Type").unwrap())
        .unwrap();
    assert_eq!(header_values[0], mime::APPLICATION_OCTET_STREAM.to_string());
    let foo = resp.take_body().into_string().await.unwrap();
    assert_eq!(foo.as_bytes(), b"foo");
}

#[async_std::test]
async fn string_content_type() {
    use http_types::headers::HeaderName;
    use std::str::FromStr;

    let mut resp = Response::new(StatusCode::Ok).body_string("foo".to_string());
    let header_values = resp
        .header(&HeaderName::from_str("Content-Type").unwrap())
        .unwrap();
    assert_eq!(header_values[0], mime::TEXT_PLAIN_UTF_8.to_string());
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
