//! Project repository — persistence layer for project CRUD operations.
//!
//! All methods are async and take a `&PgPool` as the first argument,
//! keeping the repository stateless (per cloud-native / domain-web constraints).
//! Operations that modify data (`create`, `delete`) write an audit log entry
//! atomically within the same transaction.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use alpe_core::jurisdiction::Jurisdiction;

use crate::error::RepositoryError;

/// A project row as stored in the database.
///
/// This is the read-side representation returned by all repository queries.
/// It maps 1:1 to the `projects` table columns.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProjectRow {
    /// Unique project identifier.
    pub id: Uuid,
    /// Project name (unique per owner).
    pub name: String,
    /// The jurisdiction this project is bound to.
    pub jurisdiction: Jurisdiction,
    /// The user who owns this project.
    pub owner_id: Uuid,
    /// When the project was created.
    pub created_at: DateTime<Utc>,
    /// When the project was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Repository for project persistence operations.
///
/// `ProjectRepository` is a unit struct with associated functions — it holds no
/// state. The database connection pool is passed as a parameter to each method,
/// following the stateless-service pattern recommended by the `domain-web` and
/// `domain-cloud-native` skills.
///
/// # Transactions
///
/// `create` and `delete` use explicit transactions to guarantee that the project
/// mutation and its corresponding audit log entry are written atomically.
///
/// # Errors
///
/// All methods return [`RepositoryError`] which categorises failures as:
/// - `NotFound` — the project does not exist
/// - `Conflict` — a uniqueness constraint was violated
/// - `Database` — a low-level sqlx error
pub struct ProjectRepository;

impl ProjectRepository {
    /// Creates a new project and records a `"project.created"` audit log entry.
    ///
    /// The project is inserted within a transaction together with its audit log
    /// entry, so either both succeed or neither does.
    ///
    /// # Errors
    ///
    /// - [`RepositoryError::Conflict`] if a project with the same name already
    ///   exists for this owner.
    /// - [`RepositoryError::Database`] on any other database failure.
    #[tracing::instrument(skip(pool), fields(project_name = %name, owner = %owner_id))]
    pub async fn create(
        pool: &PgPool,
        name: &str,
        jurisdiction: Jurisdiction,
        owner_id: Uuid,
    ) -> Result<ProjectRow, RepositoryError> {
        let mut tx = pool.begin().await?;

        let row = sqlx::query_as::<_, ProjectRow>(
            r"
            INSERT INTO projects (name, jurisdiction, owner_id)
            VALUES ($1, $2, $3)
            RETURNING id, name, jurisdiction, owner_id, created_at, updated_at
            ",
        )
        .bind(name)
        .bind(jurisdiction)
        .bind(owner_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err)
                if db_err.constraint() == Some("uq_projects_owner_name") =>
            {
                RepositoryError::conflict(format!(
                    "project with name '{name}' already exists for this owner"
                ))
            }
            _ => RepositoryError::Database(e),
        })?;

        // Audit log entry
        sqlx::query(
            r"
            INSERT INTO audit_log (project_id, user_id, action, resource_type, resource_id)
            VALUES ($1, $2, 'project.created', 'project', $3)
            ",
        )
        .bind(row.id)
        .bind(owner_id)
        .bind(row.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        tracing::info!(project_id = %row.id, "project created");
        Ok(row)
    }

    /// Returns a project by its unique identifier, or `None` if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Database`] on any database failure.
    #[tracing::instrument(skip(pool))]
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ProjectRow>, RepositoryError> {
        let row = sqlx::query_as::<_, ProjectRow>(
            "SELECT id, name, jurisdiction, owner_id, created_at, updated_at FROM projects WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    /// Lists all projects owned by the given user.
    ///
    /// Returns an empty `Vec` if the user has no projects (this is not an error).
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Database`] on any database failure.
    #[tracing::instrument(skip(pool))]
    pub async fn list_by_owner(
        pool: &PgPool,
        owner_id: Uuid,
    ) -> Result<Vec<ProjectRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, ProjectRow>(
            r"
            SELECT id, name, jurisdiction, owner_id, created_at, updated_at
            FROM projects
            WHERE owner_id = $1
            ORDER BY created_at ASC
            ",
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    /// Deletes a project and records a `"project.deleted"` audit log entry.
    ///
    /// The deletion and audit log entry are written within a single transaction.
    ///
    /// # Errors
    ///
    /// - [`RepositoryError::NotFound`] if no project with the given ID exists.
    /// - [`RepositoryError::Database`] on any other database failure.
    #[tracing::instrument(skip(pool))]
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), RepositoryError> {
        let mut tx = pool.begin().await?;

        let result = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        // Audit log entry
        sqlx::query(
            r"
            INSERT INTO audit_log (project_id, user_id, action, resource_type, resource_id)
            VALUES ($1, $2, 'project.deleted', 'project', $3)
            ",
        )
        .bind(id)
        .bind(user_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        tracing::info!(%id, "project deleted");
        Ok(())
    }
}
