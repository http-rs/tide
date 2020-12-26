// https://github.com/asecurityteam/loadshed

use crate::{Middleware, Next, Request, Response};

use async_std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use pid_lite::Controller;

/// Proportional request rejection based on load metrics.
///
/// # What is this purpose of this?
///
/// This middleware starts rejecting requests once a threshold has been reached
/// telling clients to try again. This enables a service to cope with sudden
/// increases in traffic without going down.
///
/// # How does this work?
///
/// This middleware sets a target number of requests it can process concurrently.
/// Once capacity has been achieved it starts yielding back `503: service
/// unavailable` in order to shed load. Responses also include a `Retry-After`
/// header which sets a time in the future when to retry again
///
/// Internally capacity is governed through a [PID
/// controller](https://en.wikipedia.org/wiki/PID_controller). This enables
/// gradually scaling up load which prevents suddenly overwhelming the system and
/// potentially crashing.
///
/// # What should clients implement?
///
/// Ideally a client will understand it can retry `503` requests, and will retry
/// with the `Retry-After` value from the response after the timeout has elapsed.
///
/// # What other mitigations can be applied?
///
/// Always use a CDN which provides DDoS protection, and correctly configure your
/// firewalls. Beyond that there are many kinds of rate limiters, and
/// [Stripe has an excellent blog post](https://stripe.com/blog/rate-limiters)
/// listing which exist and how to reason about them.
#[derive(Debug)]
pub struct LoadShedMiddleware {
    inner: RwLock<Inner>,
    /// The current amount of requests in flight.
    counter: Arc<()>,
    // TODO: create another Arc to count how many instance of this middleware exist.
}

#[derive(Debug)]
struct Inner {
    /// The PID controller governing the load.
    controller: Controller,
    /// The target number of concurrent requests we can have before we start
    /// shedding load.
    current_target: f64,
    last_time: Instant,
}

impl LoadShedMiddleware {
    /// Create a new instance of `LoadShedMiddleware`.
    ///
    /// `target` defines the desired amount of concurrent requests we want to
    /// reach in order to optimize throughput on this service.
    pub fn new(target: f64) -> Self {
        Self::with_gain(target, 0.5, 0.1, 0.2)
    }

    /// Create a new instance of `LoadShedMiddleware` with custom tuning parameters.
    /// TODO: pass a callback so it can be "dark applied".
    /// TODO: consider "dark applying" a first-class mode that we should enable
    /// TODO: apply `log` logs.
    pub fn with_gain(target: f64, p_gain: f64, i_gain: f64, d_gain: f64) -> Self {
        let current_target = 0.0;
        let mut controller = Controller::new(target, p_gain, i_gain, d_gain);
        let _correction = controller.update(current_target);
        Self {
            inner: RwLock::new(Inner {
                controller,
                current_target,
                last_time: Instant::now(),
            }),
            counter: Arc::new(()),
        }
    }

    /// Get the target value.
    pub async fn target(&self) -> f64 {
        let guard = self.inner.read().await;
        guard.controller.target()
    }

    /// Set the target value.
    pub async fn set_target(&mut self, target: f64) {
        let mut guard = self.inner.write().await;
        guard.controller.set_target(target);
    }
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for LoadShedMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> crate::Result {
        // Init the middleware's request state.
        let count_guard = self.counter.clone();
        let current_count = Arc::strong_count(&count_guard);

        // Update the PID controller if needed.
        let now = Instant::now();
        let last_time = self.inner.read().await.last_time;
        let elapsed = now.duration_since(last_time);
        drop(last_time);
        if elapsed > Duration::from_secs(1) {
            let mut guard = self.inner.write().await;
            guard.last_time = now;
            let correction = guard.controller.update(current_count as f64);
            guard.current_target += correction;
        }

        // Check whether a 503 should be returned.
        let guard = self.inner.read().await;
        if current_count > guard.current_target as usize {
            println!(
                "Load shedding middleware engaged. target count: {}, current count: {}",
                guard.controller.target(),
                current_count
            );
            return Ok(Response::new(503));
        }

        // Finish running the request.
        let res = next.run(req).await;
        drop(count_guard);
        Ok(res)
    }
}
