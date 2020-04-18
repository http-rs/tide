use async_std::{io, task};
use http_types::StatusCode;

static STATIC_DATA: &[u8] = &[4, 2];

fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let mut app = tide::new();

        app.at("/bytes").get(|_req: tide::Request<()>| async move {
            // For static data you can pass it directly:
            let _ = tide::Response::new(StatusCode::Ok).body(STATIC_DATA);

            // For non-static data you need to use a `Cursor`:
            let dynamically_generated_data = vec![4, 2];
            tide::Response::new(StatusCode::Ok).body(io::Cursor::new(dynamically_generated_data))
        });
        app.listen("0.0.0.0:8000").await?;
        Ok(())
    })
}
