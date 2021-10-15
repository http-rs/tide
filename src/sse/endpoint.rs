use crate::http::{mime, Body, StatusCode};
use crate::sse::Sender;
use crate::{log, State};
use crate::{Endpoint, Request, Response, Result};

use async_std::future::Future;
use async_std::io::BufReader;
use async_std::task;

use std::marker::PhantomData;
use std::sync::Arc;

/// Create an endpoint that can handle SSE connections.
pub fn endpoint<F, Fut, ServerState>(handler: F) -> SseEndpoint<F, Fut, ServerState>
where
    ServerState: Clone + Send + Sync + 'static,
    F: Fn(Request, State<ServerState>, Sender) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    SseEndpoint {
        handler: Arc::new(handler),
        __state: PhantomData,
    }
}

/// An endpoint that can handle SSE connections.
#[derive(Debug)]
pub struct SseEndpoint<F, Fut, ServerState>
where
    ServerState: Clone + Send + Sync + 'static,
    F: Fn(Request, State<ServerState>, Sender) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    handler: Arc<F>,
    __state: PhantomData<ServerState>,
}

#[async_trait::async_trait]
impl<F, Fut, ServerState> Endpoint<ServerState> for SseEndpoint<F, Fut, ServerState>
where
    ServerState: Clone + Send + Sync + 'static,
    F: Fn(Request, State<ServerState>, Sender) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    async fn call(&self, req: Request, state: State<ServerState>) -> Result<Response> {
        let handler = self.handler.clone();
        let (sender, encoder) = async_sse::encode();
        task::spawn(async move {
            let sender = Sender::new(sender);
            if let Err(err) = handler(req, state, sender).await {
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
