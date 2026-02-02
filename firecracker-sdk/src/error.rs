/// Errors returned by the Firecracker SDK.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API error: {0}")]
    Api(#[from] firecracker_api::Error<firecracker_api::types::Error>),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("missing required configuration: {0}")]
    MissingConfig(&'static str),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
