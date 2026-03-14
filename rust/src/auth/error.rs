use std::fmt;

/// SDK authentication error type.
///
/// Used in the auth contract to represent errors
/// from policy checks, user lookups, and other auth
/// operations.
#[derive(Debug)]
pub enum AuthError {
    /// The caller does not have permission.
    Forbidden(String),
    /// The requested resource was not found.
    NotFound(String),
    /// Bad request / validation error.
    BadRequest(String),
    /// The caller is not authenticated.
    Unauthorized(String),
    /// Internal / unexpected error.
    Internal(String),
    /// HTTP transport error.
    Http(reqwest::Error),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Forbidden(msg) => write!(f, "Forbidden: {msg}"),
            Self::NotFound(msg) => write!(f, "NotFound: {msg}"),
            Self::BadRequest(msg) => write!(f, "BadRequest: {msg}"),
            Self::Unauthorized(msg) => {
                write!(f, "Unauthorized: {msg}")
            }
            Self::Internal(msg) => write!(f, "Internal: {msg}"),
            Self::Http(e) => write!(f, "Http: {e}"),
        }
    }
}

impl std::error::Error for AuthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

/// Result alias for auth operations.
pub type AuthResult<T> = Result<T, AuthError>;
