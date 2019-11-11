// use futures::executor::block_on;
// use http_service::Body;
// use http_service_mock::make_server;
// use tide::*;

// #[derive(Deserialize)]
// struct Params {
//     msg: String,
// }

// async fn handler(cx: crate::Request<()>) -> Result<String, Error> {
//     let p = cx.url_query::<Params>()?;
//     Ok(p.msg)
// }

// fn app() -> crate::Server<()> {
//     let mut app = crate::Server::new();
//     app.at("/").get(handler);
//     app
// }

// #[test]
// fn successfully_deserialize_query() {
//     let app = app();
//     let mut server = make_server(app.into_http_service()).unwrap();
//     let req = http::Request::get("/?msg=Hello")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 200);
//     let body = block_on(res.into_body().into_vec()).unwrap();
//     assert_eq!(&*body, &*b"Hello");
// }

// #[test]
// fn unsuccessfully_deserialize_query() {
//     let app = app();
//     let mut server = make_server(app.into_http_service()).unwrap();
//     let req = http::Request::get("/").body(Body::empty()).unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 400);
// }

// #[test]
// fn malformatted_query() {
//     let app = app();
//     let mut server = make_server(app.into_http_service()).unwrap();
//     let req = http::Request::get("/?error=should_fail")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 400);
// }
