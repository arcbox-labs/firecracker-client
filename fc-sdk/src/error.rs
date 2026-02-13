use std::fmt;
use std::path::PathBuf;
use std::process::ExitStatus;

/// Errors returned by the Firecracker SDK.
#[derive(Debug)]
pub enum Error {
    /// API error with error body from Firecracker.
    Api(fc_api::Error<fc_api::types::Error>),

    /// API error without error body (e.g., for endpoints with only default response).
    ApiNoBody(fc_api::Error<()>),

    /// HTTP/network error.
    Http(reqwest::Error),

    /// I/O error.
    Io(std::io::Error),

    /// Failed to spawn a process.
    SpawnFailed(std::io::Error),

    /// Timed out waiting for the API socket to become available.
    SocketTimeout(PathBuf),

    /// The process exited unexpectedly.
    ProcessExited(Option<ExitStatus>),

    /// Bundled binary cannot be found.
    BundledBinaryNotFound {
        binary: &'static str,
        searched: Vec<PathBuf>,
    },

    /// Bundled binary exists but is not executable.
    BundledBinaryNotExecutable(PathBuf),

    /// Bundled SHA256 string format is invalid.
    BundledInvalidSha256 {
        binary: &'static str,
        sha256: String,
    },

    /// Bundled binary checksum mismatched.
    BundledChecksumMismatch {
        binary: &'static str,
        path: PathBuf,
        expected: String,
        actual: String,
    },

    /// Unsupported platform for Firecracker release-based bundled mode.
    BundledUnsupportedPlatform { os: String, arch: String },

    /// Invalid Firecracker release version.
    BundledInvalidReleaseVersion(String),

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
            Self::Io(e) => Some(e),
            Self::SpawnFailed(e) => Some(e),
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
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::SpawnFailed(e) => write!(f, "failed to spawn process: {e}"),
            Self::SocketTimeout(path) => {
                write!(f, "timed out waiting for socket: {}", path.display())
            }
            Self::ProcessExited(Some(status)) => {
                write!(f, "process exited unexpectedly: {status}")
            }
            Self::ProcessExited(None) => write!(f, "process exited unexpectedly"),
            Self::BundledBinaryNotFound { binary, searched } => {
                write!(
                    f,
                    "bundled binary not found: {binary}; searched: {}",
                    searched
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Self::BundledBinaryNotExecutable(path) => {
                write!(f, "bundled binary is not executable: {}", path.display())
            }
            Self::BundledInvalidSha256 { binary, sha256 } => {
                write!(f, "invalid SHA256 for {binary}: {sha256}")
            }
            Self::BundledChecksumMismatch {
                binary,
                path,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "checksum mismatch for {binary} ({}): expected {expected}, got {actual}",
                    path.display()
                )
            }
            Self::BundledUnsupportedPlatform { os, arch } => {
                write!(
                    f,
                    "unsupported platform for bundled release mode: {os}-{arch}; supported: linux-x86_64, linux-aarch64"
                )
            }
            Self::BundledInvalidReleaseVersion(version) => {
                write!(
                    f,
                    "invalid Firecracker release version: {version}; expected vX.Y.Z"
                )
            }
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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
