mod test_utils;
use test_utils::ServerTestingExt;
use tide::http::{headers, mime};
use tide::{Response, StatusCode};

#[async_std::test]
async fn test_status() {
    let mut resp = Response::new(StatusCode::NotFound);
    resp.set_body("foo");
    assert_eq!(resp.status(), StatusCode::NotFound);
    let body = resp.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"foo");
}

#[async_std::test]
async fn byte_vec_content_type() {
    use async_std::io::Cursor;
    use tide::Body;

    let mut resp = Response::new(StatusCode::Ok);
    resp.set_body(Body::from_reader(Cursor::new("foo"), None));

    assert_eq!(resp[headers::CONTENT_TYPE], mime::BYTE_STREAM.to_string());
    let body = resp.take_body().into_bytes().await.unwrap();
    assert_eq!(body, b"foo");
}

#[async_std::test]
async fn string_content_type() {
    let mut resp = Response::new(StatusCode::Ok);
    resp.set_body("foo");

    assert_eq!(resp[headers::CONTENT_TYPE], mime::PLAIN.to_string());
    let body = resp.take_body().into_string().await.unwrap();
    assert_eq!(body, "foo");
}

#[async_std::test]
async fn json_content_type() -> tide::Result<()> {
    use std::collections::BTreeMap;
    use tide::Body;

    let mut app = tide::new();
    app.at("/json_content_type").get(|_| async {
        let mut map = BTreeMap::new();
        map.insert(Some("a"), 2);
        map.insert(Some("b"), 4);
        map.insert(None, 6);
        let mut resp = Response::new(StatusCode::Ok);
        resp.set_body(Body::from_json(&map)?);
        Ok(resp)
    });

    let mut resp = app.get("/json_content_type").await?;
    assert_eq!(resp.status(), StatusCode::InternalServerError);
    assert_eq!(resp.body_string().await.unwrap(), "");

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

    Ok(())
}

#[test]
fn from_response() {
    let msg = "This is an error";

    let error = tide::Error::from_str(StatusCode::NotFound, msg);
    let mut res: Response = error.into();

    assert!(res.error().is_some());
    // Ensure we did not consume the error
    assert!(res.error().is_some());

    assert_eq!(res.error().unwrap().status(), StatusCode::NotFound);
    assert_eq!(res.error().unwrap().to_string(), msg);

    res.take_error();
    assert!(res.error().is_none());
}
