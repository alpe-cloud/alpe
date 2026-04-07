//! Database repository error types.
//!
//! Errors are categorized by audience (following the m13-domain-error pattern):
//! - [`RepositoryError::NotFound`] — user-facing: the requested resource does not exist
//! - [`RepositoryError::Conflict`] — user-facing: a uniqueness constraint was violated
//! - [`RepositoryError::Database`] — internal/transient: wraps low-level sqlx errors

/// Errors returned by repository operations.
///
/// These are designed to be mapped into HTTP status codes by the API layer:
/// - `NotFound` → 404
/// - `Conflict` → 409
/// - `Database` → 500
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RepositoryError {
    /// The requested resource was not found.
    #[error("resource not found")]
    NotFound,

    /// A uniqueness constraint was violated (e.g. duplicate project name for the same owner).
    #[error("conflict: {message}")]
    Conflict {
        /// Human-readable description of the conflict.
        message: String,
    },

    /// A low-level database error occurred.
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

impl RepositoryError {
    /// Creates a new conflict error with the given message.
    pub(crate) fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }
}
