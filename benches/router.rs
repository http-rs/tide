use criterion::{black_box, criterion_group, criterion_main, Criterion};
use http_types::Method;
use tide::router::Router;

fn criterion_benchmark(c: &mut Criterion) {
    let mut router = Router::<()>::new();
    router.add(
        "hello",
        Method::Get,
        Box::new(|_| async move { Ok("hello world") }),
    );

    c.bench_function("route-match", |b| {
        b.iter(|| black_box(router.route("/hello", Method::Get)))
    });

    c.bench_function("route-root", |b| {
        b.iter(|| black_box(router.route("", Method::Get)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
