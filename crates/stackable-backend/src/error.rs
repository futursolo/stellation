use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerAppError {
    #[error("failed to parse queries")]
    Queries(#[from] serde_urlencoded::de::Error),
}

pub type ServerAppResult<T> = Result<T, ServerAppError>;
