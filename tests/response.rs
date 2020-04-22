// use futures::executor::block_on;
use tide::{IntoResponse, Response};

#[test]
fn into_response_for_result_does_not_panic_if_error_case_maps_to_200() {
    struct DummyResponse {}

    impl IntoResponse for DummyResponse {
        fn into_response(self) -> Response {
            Response::new(200)
        }
    }

    let result: Result<String, _> = Err(DummyResponse {});
    assert!(result.into_response().status().is_success());
}

// #[test]
// fn test_status() {
//     let resp = "foo"
//         .with_status(http::status::StatusCode::NOT_FOUND)
//         .into_response();
//     assert_eq!(resp.status(), http::status::StatusCode::NOT_FOUND);
//     assert_eq!(block_on(resp.into_body().into_vec()).unwrap(), b"foo");
// }

// #[test]
// fn byte_vec_content_type() {
//     let resp = String::from("foo").into_bytes().into_response();
//     assert_eq!(resp.headers()["Content-Type"], "application/octet-stream");
//     assert_eq!(block_on(resp.into_body().into_vec()).unwrap(), b"foo");
// }

// #[test]
// fn string_content_type() {
//     let resp = String::from("foo").into_response();
//     assert_eq!(resp.headers()["Content-Type"], "text/plain; charset=utf-8");
//     assert_eq!(block_on(resp.into_body().into_vec()).unwrap(), b"foo");
// }

// #[test]
// fn json_content_type() {
//     use std::collections::BTreeMap;

//     let mut map = BTreeMap::new();
//     map.insert(Some("a"), 2);
//     map.insert(Some("b"), 4);
//     map.insert(None, 6);

//     let resp = json(map);
//     assert_eq!(
//         resp.status(),
//         http::status::StatusCode::INTERNAL_SERVER_ERROR
//     );
//     assert_eq!(block_on(resp.into_body().into_vec()).unwrap(), b"");

//     let mut map = BTreeMap::new();
//     map.insert("a", 2);
//     map.insert("b", 4);
//     map.insert("c", 6);

//     let resp = json(map);
//     assert_eq!(resp.status(), http::status::StatusCode::OK);
//     assert_eq!(
//         block_on(resp.into_body().into_vec()).unwrap(),
//         br##"{"a":2,"b":4,"c":6}"##
//     );
// }
