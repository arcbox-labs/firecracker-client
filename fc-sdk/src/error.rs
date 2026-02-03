use std::fmt;

/// Errors returned by the Firecracker SDK.
#[derive(Debug)]
pub enum Error {
    /// API error with error body from Firecracker.
    Api(fc_api::Error<fc_api::types::Error>),

    /// API error without error body (e.g., for endpoints with only default response).
    ApiNoBody(fc_api::Error<()>),

    /// HTTP/network error.
    Http(reqwest::Error),

    /// Missing required configuration.
    MissingConfig(&'static str),

    /// Other error.
    Other(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Api(e) => Some(e),
            Self::ApiNoBody(e) => Some(e),
            Self::Http(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api(e) => write!(f, "API error: {e}"),
            Self::ApiNoBody(e) => write!(f, "API error: {e}"),
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::MissingConfig(field) => write!(f, "missing required configuration: {field}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<fc_api::Error<fc_api::types::Error>> for Error {
    fn from(err: fc_api::Error<fc_api::types::Error>) -> Self {
        Self::Api(err)
    }
}

impl From<fc_api::Error<()>> for Error {
    fn from(err: fc_api::Error<()>) -> Self {
        Self::ApiNoBody(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
