//! # alpe-test
//!
//! Shared test infrastructure for the Alpe platform.
//!
//! This crate is a **dev-dependency only** — it is never compiled into any
//! shipped binary. It provides reusable test utilities used across all crates:
//!
//! - [`TestDb`] — spins up an ephemeral Postgres via testcontainers, runs migrations
//! - `TestApi` — boots the axum server on a random port with a `TestDb` (future)
//! - `TestUser` — factory for creating test users with specific roles (future)
//! - `Fixture` — trait for seeding test data (future)
//! - Assert helpers for API response validation (future)

use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

/// An ephemeral Postgres database backed by testcontainers.
///
/// `TestDb` spins up a real Postgres container, creates a connection pool,
/// and runs all `alpe-db` migrations. Each test gets its own isolated database.
///
/// # Examples
///
/// ```rust,ignore
/// #[tokio::test]
/// async fn my_test() {
///     let db = TestDb::new().await;
///     let pool = db.pool();
///     // use pool for queries…
/// }
/// ```
///
/// The container and database are dropped automatically when `TestDb` goes out
/// of scope.
pub struct TestDb {
    pool: PgPool,
    // Hold the container handle so it stays alive for the duration of the test.
    _container: testcontainers::ContainerAsync<Postgres>,
}

impl TestDb {
    /// Spins up a new Postgres container and runs all migrations.
    ///
    /// # Panics
    ///
    /// Panics if the container cannot be started or migrations fail.
    /// This is intentional — test infrastructure failures should fail loudly.
    #[allow(clippy::expect_used)]
    pub async fn new() -> Self {
        let container = Postgres::default()
            .start()
            .await
            .expect("failed to start Postgres container");

        let host_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("failed to get Postgres port");

        let connection_string =
            format!("postgres://postgres:postgres@127.0.0.1:{host_port}/postgres");

        let pool = PgPool::connect(&connection_string)
            .await
            .expect("failed to connect to test Postgres");

        alpe_db::run_migrations(&pool)
            .await
            .expect("failed to run migrations");

        Self {
            pool,
            _container: container,
        }
    }

    /// Returns a reference to the connection pool.
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Creates a test user and returns their UUID.
    ///
    /// This inserts a minimal user row with a dummy password hash, suitable
    /// for tests that need an `owner_id` foreign key.
    #[allow(clippy::expect_used)]
    pub async fn create_test_user(&self, email: &str, name: &str) -> Uuid {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO users (email, name, password_hash) VALUES ($1, $2, '$argon2id$test_hash') RETURNING id",
        )
        .bind(email)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .expect("failed to create test user");

        row.0
    }
}
