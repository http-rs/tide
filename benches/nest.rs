use criterion::{black_box, criterion_group, criterion_main, Criterion};
use http_types::{Method, Request, Response, Url};

fn criterion_benchmark(c: &mut Criterion) {
    let mut app = tide::new();
    app.at("/x").get(|_| async { Ok("X") });
    app.at("/x/y").get(|_| async { Ok("Y") });
    app.at("/x/y/z").get(|_| async { Ok("Z") });

    let route = Url::parse("https://example.com/x/y/z").unwrap();
    let req = Request::new(Method::Get, route);
    c.bench_function("plain", |b| {
        b.iter(|| black_box(app.respond::<_, Response>(req.clone())));
    });

    let mut appz = tide::new();
    appz.at("/z").get(|_| async { Ok("Z") });
    
    let mut appy = tide::new();
    appy.at("/y").nest(appz);

    let mut appx = tide::new();
    appx.at("/x").nest(appy);

    let route = Url::parse("https://example.com/x/y/z").unwrap();
    let req = Request::new(Method::Get, route);
    c.bench_function("nested", |b| {
        b.iter(|| black_box(appx.respond::<_, Response>(req.clone())));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
