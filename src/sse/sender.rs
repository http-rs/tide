/// An SSE message sender.
#[derive(Debug)]
pub struct Sender {
    sender: async_sse::Sender,
}

impl Sender {
    /// Create a new instance of `Sender`.
    pub(crate) fn new(sender: async_sse::Sender) -> Self {
        Self { sender }
    }

    /// Send data from the SSE channel.
    ///
    /// Each message constists of a "name" and "data".
    pub async fn send(&self, name: &str, data: impl AsRef<str>, id: Option<&str>) {
        self.sender.send(name, data.as_ref(), id).await;
    }
}
