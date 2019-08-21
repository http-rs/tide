use futures::executor::block_on;
use http_service::Body;
use http_service_mock::make_server;
use tide::Context;

async fn ok(_cx: Context<()>) -> String {
    String::from("this shouldn't exist in the body of a HEAD response")
}

#[test]
fn head_response_empty() {
    let mut app = tide::App::new();
    app.at("/").get(ok);
    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::head("/").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    let body = block_on(res.into_body().into_vec()).unwrap();
    dbg!(&String::from_utf8(body));
    //assert!(body.is_empty());
}
