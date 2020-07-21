use async_std::prelude::*;
use std::time::Duration;

mod test_utils;

#[async_std::test]
async fn start_server_log() {
    let mut logger = logtest::start();

    let port = test_utils::find_port().await;
    let app = tide::new();
    let res = app
        .listen(("localhost", port))
        .timeout(Duration::from_millis(60))
        .await;
    assert!(res.is_err());

    let record = logger
        .find(|rec| rec.args().starts_with("Server listening"))
        .unwrap();
    assert_eq!(
        record.args(),
        format!("Server listening on http://[::1]:{}", port)
    );
}
