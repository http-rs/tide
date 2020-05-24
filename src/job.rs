//! Jobs.

use std::{future::Future, sync::Arc};

use async_std::task;

/// Job context, that handles state
#[derive(Debug)]
pub struct JobContext<State> {
    state: Arc<State>
}

impl<State> JobContext<State> {
    pub(crate) fn new(state: Arc<State>) -> Self {
        Self {
            state
        }
    }

    /// Access app-global state
    pub fn state(&self) -> &State {
        &self.state
    }
}

/// Job trait for handling background tasks.
pub trait Job<State>: 'static + Send + Sync {
    /// Asynchronously execute job.
    fn handle(
        &self,
        ctx: JobContext<State>,
    );
}

impl<State, F, Fut> Job<State> for F
where
    F: Fn(JobContext<State>) -> Fut
        + Send
        + Sync
        + 'static,
    Fut: Future<Output = ()> + Send + 'static
{
    fn handle(
        &self,
        ctx: JobContext<State>,
    ) {
        let fut = (self)(ctx);
        task::spawn(async move {
            fut.await;
        });
    }
}