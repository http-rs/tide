/// Information about the `listener`.
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    connection: String,
}

impl ConnectionInfo {
    /// Create a new instance of `ConnectionInfo`.
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
