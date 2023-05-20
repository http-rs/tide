use crate::{utils::async_trait, Middleware, Next, Request};

/// Sets data onto the request extensions
#[derive(Debug)]
pub struct StateMiddleware<T: Clone + Send + Sync + 'static> {
    data: T,
}

impl<T: Clone + Send + Sync + 'static> StateMiddleware<T> {
    /// Creates a new state middleware with the provided state
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[async_trait]
impl<T: Clone + Send + Sync + 'static> Middleware for StateMiddleware<T> {
    async fn handle(&self, mut request: Request, next: Next) -> crate::Result {
        request.set_ext(self.data.clone());
        let mut response = next.run(request).await;
        response.set_ext(self.data.clone());
        Ok(response)
    }
}
