use async_std::task;
extern crate env_logger;
#[macro_use] extern crate tera;
use tera::Tera;
use tide::{Request, Response};

async fn index(_req: Request<Tera>)-> Response{
    let mut res= Response::new(200);
    let mut context = tera::Context::new();
    context.insert("name", &"HT");
    let tmpl = _req.state().render(&"index.html", &context);
    res = res.body_string(tmpl.unwrap());
    res = res.set_header("content-type", "text/html");
    res
}

fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let teradir = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        let mut app = tide::with_state(teradir);
        app.at("/").all(index);
        app.listen("127.0.0.1:8080").await?;
        Ok(())
    })
}
