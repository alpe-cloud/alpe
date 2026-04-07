//! # alpe-db
//!
//! Database migrations, connection pooling, and repository layer for the Alpe platform.
//!
//! This crate owns all persistence concerns:
//! - SQL migrations (embedded via [`sqlx::migrate!`])
//! - Repository implementations for domain entities
//! - A typed [`RepositoryError`] hierarchy
//!
//! ## Architecture
//!
//! Repositories are stateless unit structs with associated async functions that
//! accept a `&PgPool`. This keeps the data layer aligned with cloud-native
//! (stateless, horizontally scalable) and web (shared `Arc<PgPool>`) patterns.
//!
//! ## Migrations
//!
//! Migrations are embedded into the binary at compile time. Call [`run_migrations`]
//! at application startup to apply them:
//!
//! ```rust,ignore
//! alpe_db::run_migrations(&pool).await?;
//! ```

/// Database repository error types.
pub mod error;

/// Project repository — CRUD operations for the `projects` table.
pub mod project;

pub use error::RepositoryError;
pub use project::{ProjectRepository, ProjectRow};

/// Runs all pending database migrations against the given pool.
///
/// This embeds the migration SQL files from the `migrations/` directory at
/// compile time, so the binary is self-contained — no external migration files
/// are needed at runtime.
///
/// # Errors
///
/// Returns a [`sqlx::migrate::MigrateError`] if any migration fails to apply.
pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
