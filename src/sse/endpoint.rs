use crate::http::{mime, Body, StatusCode};
use crate::log;
use crate::sse::Sender;
use crate::{Endpoint, Request, Response, Result};

use async_std::future::Future;
use async_std::io::BufReader;
use async_std::task;

use std::marker::PhantomData;
use std::sync::Arc;

/// Create an endpoint that can handle SSE connections.
pub fn endpoint<F, Fut>(handler: F) -> SseEndpoint<F, Fut>
where
    F: Fn(Request, Sender) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    SseEndpoint {
        handler: Arc::new(handler),
    }
}

/// An endpoint that can handle SSE connections.
#[derive(Debug)]
pub struct SseEndpoint<F, Fut>
where
    F: Fn(Request, Sender) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    handler: Arc<F>,
}

#[async_trait::async_trait]
impl<F, Fut> Endpoint for SseEndpoint<F, Fut>
where
    F: Fn(Request, Sender) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    async fn call(&self, req: Request) -> Result<Response> {
        let handler = self.handler.clone();
        let (sender, encoder) = async_sse::encode();
        task::spawn(async move {
            let sender = Sender::new(sender);
            if let Err(err) = handler(req, sender).await {
                log::error!("SSE handler error: {:?}", err);
            }
        });

        // Perform the handshake as described here:
        // https://html.spec.whatwg.org/multipage/server-sent-events.html#sse-processing-model
        let mut res = Response::new(StatusCode::Ok);
        res.insert_header("Cache-Control", "no-cache");
        res.set_content_type(mime::SSE);

        let body = Body::from_reader(BufReader::new(encoder), None);
        res.set_body(body);

        Ok(res)
    }
}
