use async_std::task::block_on;
// use http_service::Body;
// use http_service_mock::make_server;
use http_types::{Body, Method, StatusCode};
use tide::{http, Request, Response};

async fn add_one(cx: Request<()>) -> Result<String, tide::Error> {
    return match cx.param::<i64>("num"){
        Ok(num) => Ok((num + 1).to_string()),
        Err(err) => Err(tide::Error::new(StatusCode::BadRequest, err))
    }
}

// async fn add_two(cx: Request<()>) -> Result<String, tide::Error> {
//     let one: i64 = cx.param("one").client_err()?;
//     let two: i64 = cx.param("two").client_err()?;
//     Ok((one + two).to_string())
// }

// async fn echo_path(cx: Request<()>) -> Result<String, tide::Error> {
//     let path: String = cx.param("path").client_err()?;
//     Ok(path)
// }

// async fn echo_empty(cx: Request<()>) -> Result<String, tide::Error> {
//     let path: String = cx.param("").client_err()?;
//     Ok(path)
// }

#[async_std::test]
async fn wildcard() {
    let mut app = tide::Server::new();
    app.at("/add_one/:num").get(add_one);

    let mut req = http::Request::new(Method::Get, "http://localhost/add_one/3".parse().unwrap());
    req.set_body(Body::empty());

    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"4");

    let mut req = http::Request::new(Method::Get, "http://localhost/add_one/-7".parse().unwrap());
    req.set_body(Body::empty());

    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"-6");
}

// #[test]
// fn invalid_segment_error() {
//     let mut app = tide::Server::new();
//     app.at("/add_one/:num").get(add_one);
//     let mut server = make_server(app.into_http_service()).unwrap();

//     let req = http::Request::get("/add_one/a")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 400);
// }

// #[test]
// fn not_found_error() {
//     let mut app = tide::Server::new();
//     app.at("/add_one/:num").get(add_one);
//     let mut server = make_server(app.into_http_service()).unwrap();

//     let req = http::Request::get("/add_one/").body(Body::empty()).unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 404);
// }

// #[test]
// fn wildpath() {
//     let mut app = tide::Server::new();
//     app.at("/echo/*path").get(echo_path);
//     let mut server = make_server(app.into_http_service()).unwrap();

//     let req = http::Request::get("/echo/some_path")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 200);
//     let body = block_on(res.into_body().into_vec()).unwrap();
//     assert_eq!(&*body, &*b"some_path");

//     let req = http::Request::get("/echo/multi/segment/path")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 200);
//     let body = block_on(res.into_body().into_vec()).unwrap();
//     assert_eq!(&*body, &*b"multi/segment/path");

//     let req = http::Request::get("/echo/").body(Body::empty()).unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 404);
//     let body = block_on(res.into_body().into_vec()).unwrap();
//     assert_eq!(&*body, &*b"");
// }

// #[test]
// fn multi_wildcard() {
//     let mut app = tide::Server::new();
//     app.at("/add_two/:one/:two/").get(add_two);
//     let mut server = make_server(app.into_http_service()).unwrap();

//     let req = http::Request::get("/add_two/1/2/")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 200);
//     let body = block_on(res.into_body().into_vec()).unwrap();
//     assert_eq!(&*body, &*b"3");

//     let req = http::Request::get("/add_two/-1/2/")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 200);
//     let body = block_on(res.into_body().into_vec()).unwrap();
//     assert_eq!(&*body, &*b"1");
//     let req = http::Request::get("/add_two/1")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 404);
// }

// #[test]
// fn wild_last_segment() {
//     let mut app = tide::Server::new();
//     app.at("/echo/:path/*").get(echo_path);
//     let mut server = make_server(app.into_http_service()).unwrap();

//     let req = http::Request::get("/echo/one/two")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 200);
//     let body = block_on(res.into_body().into_vec()).unwrap();
//     assert_eq!(&*body, &*b"one");

//     let req = http::Request::get("/echo/one/two/three/four")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 200);
//     let body = block_on(res.into_body().into_vec()).unwrap();
//     assert_eq!(&*body, &*b"one");
// }

// #[test]
// fn invalid_wildcard() {
//     let mut app = tide::Server::new();
//     app.at("/echo/*path/:one/").get(echo_path);
//     let mut server = make_server(app.into_http_service()).unwrap();

//     let req = http::Request::get("/echo/one/two")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 404);
// }

// #[test]
// fn nameless_wildcard() {
//     let mut app = tide::Server::new();
//     app.at("/echo/:").get(|_| async move { "" });

//     let mut server = make_server(app.into_http_service()).unwrap();

//     let req = http::Request::get("/echo/one/two")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 404);

//     let req = http::Request::get("/echo/one").body(Body::empty()).unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 200);
// }

// #[test]
// fn nameless_internal_wildcard() {
//     let mut app = tide::Server::new();
//     app.at("/echo/:/:path").get(echo_path);
//     let mut server = make_server(app.into_http_service()).unwrap();

//     let req = http::Request::get("/echo/one").body(Body::empty()).unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 404);

//     let req = http::Request::get("/echo/one/two")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 200);
//     let body = block_on(res.into_body().into_vec()).unwrap();
//     assert_eq!(&*body, &*b"two");

//     let req = http::Request::get("/echo/one/two")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 200);
//     let body = block_on(res.into_body().into_vec()).unwrap();
//     assert_eq!(&*body, &*b"two");
// }

// #[test]
// fn nameless_internal_wildcard2() {
//     let mut app = tide::Server::new();
//     app.at("/echo/:/:path").get(echo_empty);
//     let mut server = make_server(app.into_http_service()).unwrap();

//     let req = http::Request::get("/echo/one/two")
//         .body(Body::empty())
//         .unwrap();
//     let res = server.simulate(req).unwrap();
//     assert_eq!(res.status(), 200);
//     let body = block_on(res.into_body().into_vec()).unwrap();
//     assert_eq!(&*body, &*b"one");
// }
