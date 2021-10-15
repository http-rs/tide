use crate::http::{mime, Body, StatusCode};
use crate::{log, State};
use crate::{Request, Response, Result};

use super::Sender;

use async_std::future::Future;
use async_std::io::BufReader;
use async_std::task;

/// Upgrade an existing HTTP connection to an SSE connection.
pub fn upgrade<F, Fut, ServerState>(req: Request, state: State<ServerState>, handler: F) -> Response
where
    ServerState: Clone + Send + Sync + 'static,
    F: Fn(Request, State<ServerState>, Sender) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
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

    res
}
