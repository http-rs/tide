use async_std::task;
use tide::Request;

fn fib(n: usize) -> usize {
    if n == 0 || n == 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

async fn fibsum(req: Request<()>) -> tide::Result<String> {
    use std::time::Instant;
    let n: usize = req.param("n").unwrap_or(0);
    // Start a stopwatch
    let start = Instant::now();
    // Compute the nth number in the fibonacci sequence
    let fib_n = fib(n);
    // Stop the stopwatch
    let duration = Instant::now().duration_since(start).as_secs();
    // Return the answer
    let res = format!(
        "The fib of {} is {}.\nIt was computed in {} secs.\n",
        n, fib_n, duration,
    );
    Ok(res)
}
// Example: HTTP GET to http://localhost:8080/fib/42
// $ curl "http://localhost:8080/fib/42"
// The fib of 42 is 267914296.
// It was computed in 2 secs.
fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let mut app = tide::new();
        app.at("/fib/:n").get(fibsum);
        app.listen("0.0.0.0:8080").await?;
        Ok(())
    })
}
