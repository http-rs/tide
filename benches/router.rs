use criterion::{black_box, criterion_group, criterion_main, Criterion};
use http_types::{Method, Request, Response, Url};

fn criterion_benchmark(c: &mut Criterion) {
    let mut app = tide::new();
    app.at("/hello").get(|_| async { Ok("hello world") });

    let route = Url::parse("https://example.com/hello").unwrap();
    let req = Request::new(Method::Get, route);
    c.bench_function("route-match", |b| {
        b.iter(|| black_box(app.respond::<_, Response>(req.clone())));
    });

    let route = Url::parse("https://example.com").unwrap();
    let req = Request::new(Method::Get, route);
    c.bench_function("route-root", |b| {
        b.iter(|| black_box(app.respond::<_, Response>(req.clone())));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
