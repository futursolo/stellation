use thiserror::Error;

/// The error type returned by server app methods.
#[derive(Error, Debug)]
pub enum ServerAppError {
    /// failed to parse queries.
    #[error("failed to parse queries")]
    Queries(#[from] serde_urlencoded::de::Error),
}

/// The result type returned by server app methods.
pub type ServerAppResult<T> = Result<T, ServerAppError>;
