use crate::http::{Body, StatusCode};
use crate::{Request, Response, Result, Error};

use std::fmt;

use async_std::future::Future;

use async_tungstenite::WebSocketStream;
use async_tungstenite::tungstenite::protocol::Role;

pub struct Handle {
    ws: WebSocketStream<Box<dyn http_types::ReadWrite>>,
}

impl Handle {
    pub fn into_inner(self) -> WebSocketStream<Box<dyn http_types::ReadWrite>> {
        self.ws
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handle")
            .field("ws", &"...")
            .finish()
    }
}

/// Upgrade an existing HTTP connection to a websocket connection.
pub fn upgrade<F, Fut, State>(req: Request<State>, handler: F) -> Result<Response>
where
    State: Clone + Send + Sync + 'static,
    F: FnOnce(Request<State>, Handle) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + Sync + 'static,
{
    if req.version() != Some(http_types::Version::Http1_1) {
        return Err(Error::from_str(
            StatusCode::NotAcceptable,
            "Websockets are not supported on http version != 1.1.",
        ))
    }
    
    let mut res: Response = websocket_handshake::http1_1::check_request_headers(req.as_ref())
        .map_err(|err| Error::new(StatusCode::BadRequest, err))?
        .make_response().into();
    
    let body = Body::io(|io| async move {
        let ws = WebSocketStream::from_raw_socket(io, Role::Server, None).await;
        
        let handle = Handle {
            ws: ws
        };
        (handler)(req, handle).await.unwrap();
    });
    res.set_body(body);

    Ok(res)
}
