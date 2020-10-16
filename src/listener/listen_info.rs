/// Information about the `Listener`.
///
/// See [`Report`](../listener/trait.Report.html) for more.
#[derive(Debug, Clone)]
pub struct ListenInfo {
    connection: String,
}

impl ListenInfo {
    /// Create a new instance of `ListenInfo`.
    ///
    /// This method should only be called when implementing a new Tide `listener`
    /// strategy.
    pub fn new(connection: String) -> Self {
        Self { connection }
    }

    /// Get the connection string.
    pub fn connection(&self) -> &str {
        self.connection.as_str()
    }
}
