//! Integration tests for `ProjectRepository`.
//!
//! Each test spins up an ephemeral Postgres container via [`alpe_test::TestDb`]
//! and runs against real SQL — no mocking.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::doc_markdown)]

use alpe_core::jurisdiction::Jurisdiction;
use alpe_db::{ProjectRepository, RepositoryError};

use uuid::Uuid;

/// Helper: sets up a TestDb and returns the pool + a default test user.
async fn setup() -> (alpe_test::TestDb, Uuid) {
    let db = alpe_test::TestDb::new().await;
    let user_id = db.create_test_user("alice@example.com", "Alice").await;
    (db, user_id)
}

// ── CRUD basics ──

#[tokio::test]
async fn create_project_persists_and_reads_back() {
    let (db, user_id) = setup().await;
    let pool = db.pool();

    let created = ProjectRepository::create(pool, "my-project", Jurisdiction::DE, user_id)
        .await
        .expect("create should succeed");

    let fetched = ProjectRepository::get_by_id(pool, created.id)
        .await
        .expect("get_by_id should succeed")
        .expect("project should exist");

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "my-project");
    assert_eq!(fetched.jurisdiction, Jurisdiction::DE);
    assert_eq!(fetched.owner_id, user_id);
}

#[tokio::test]
async fn create_project_sets_timestamps() {
    let (db, user_id) = setup().await;
    let pool = db.pool();
    let before = chrono::Utc::now();

    let created = ProjectRepository::create(pool, "ts-project", Jurisdiction::FR, user_id)
        .await
        .expect("create should succeed");

    let after = chrono::Utc::now();

    // Timestamps should be between before and after (with some tolerance)
    assert!(
        created.created_at >= before - chrono::Duration::seconds(5),
        "created_at should be recent"
    );
    assert!(
        created.created_at <= after + chrono::Duration::seconds(5),
        "created_at should not be in the future"
    );
    assert!(
        created.updated_at >= before - chrono::Duration::seconds(5),
        "updated_at should be recent"
    );
    assert!(
        created.updated_at <= after + chrono::Duration::seconds(5),
        "updated_at should not be in the future"
    );
}

// ── Listing ──

#[tokio::test]
async fn list_projects_returns_only_owners_projects() {
    let db = alpe_test::TestDb::new().await;
    let pool = db.pool();

    let user_a = db.create_test_user("a@example.com", "User A").await;
    let user_b = db.create_test_user("b@example.com", "User B").await;

    // User A gets 2 projects
    ProjectRepository::create(pool, "a-project-1", Jurisdiction::DE, user_a)
        .await
        .expect("create a1");
    ProjectRepository::create(pool, "a-project-2", Jurisdiction::FR, user_a)
        .await
        .expect("create a2");

    // User B gets 1 project
    ProjectRepository::create(pool, "b-project-1", Jurisdiction::IT, user_b)
        .await
        .expect("create b1");

    let a_projects = ProjectRepository::list_by_owner(pool, user_a)
        .await
        .expect("list A");
    let b_projects = ProjectRepository::list_by_owner(pool, user_b)
        .await
        .expect("list B");

    assert_eq!(a_projects.len(), 2, "User A should have 2 projects");
    assert_eq!(b_projects.len(), 1, "User B should have 1 project");
}

#[tokio::test]
async fn list_projects_returns_empty_for_new_user() {
    let db = alpe_test::TestDb::new().await;
    let pool = db.pool();

    // Random UUID that has no projects
    let random_user = Uuid::new_v4();
    let projects = ProjectRepository::list_by_owner(pool, random_user)
        .await
        .expect("list should succeed");

    assert!(projects.is_empty(), "new user should have no projects");
}

// ── Get non-existent ──

#[tokio::test]
async fn get_nonexistent_project_returns_none() {
    let db = alpe_test::TestDb::new().await;
    let pool = db.pool();

    let result = ProjectRepository::get_by_id(pool, Uuid::new_v4())
        .await
        .expect("get_by_id should succeed");

    assert!(result.is_none(), "non-existent project should return None");
}

// ── Delete ──

#[tokio::test]
async fn delete_project_removes_it() {
    let (db, user_id) = setup().await;
    let pool = db.pool();

    let created = ProjectRepository::create(pool, "to-delete", Jurisdiction::AT, user_id)
        .await
        .expect("create should succeed");

    ProjectRepository::delete(pool, created.id, user_id)
        .await
        .expect("delete should succeed");

    let fetched = ProjectRepository::get_by_id(pool, created.id)
        .await
        .expect("get_by_id should succeed");

    assert!(fetched.is_none(), "deleted project should be gone");
}

#[tokio::test]
async fn delete_nonexistent_project_returns_not_found() {
    let db = alpe_test::TestDb::new().await;
    let pool = db.pool();

    let result = ProjectRepository::delete(pool, Uuid::new_v4(), Uuid::new_v4()).await;

    assert!(
        matches!(result, Err(RepositoryError::NotFound)),
        "deleting non-existent project should return NotFound"
    );
}

// ── Uniqueness constraints ──

#[tokio::test]
async fn duplicate_project_name_same_owner_rejected() {
    let (db, user_id) = setup().await;
    let pool = db.pool();

    ProjectRepository::create(pool, "unique-name", Jurisdiction::DE, user_id)
        .await
        .expect("first create should succeed");

    let result = ProjectRepository::create(pool, "unique-name", Jurisdiction::FR, user_id).await;

    assert!(
        matches!(result, Err(RepositoryError::Conflict { .. })),
        "duplicate name for same owner should be Conflict, got: {result:?}"
    );
}

#[tokio::test]
async fn duplicate_project_name_different_owner_allowed() {
    let db = alpe_test::TestDb::new().await;
    let pool = db.pool();

    let user_a = db.create_test_user("dup-a@example.com", "Dup A").await;
    let user_b = db.create_test_user("dup-b@example.com", "Dup B").await;

    let result_a = ProjectRepository::create(pool, "shared-name", Jurisdiction::DE, user_a).await;
    let result_b = ProjectRepository::create(pool, "shared-name", Jurisdiction::DE, user_b).await;

    assert!(result_a.is_ok(), "first user should succeed");
    assert!(result_b.is_ok(), "second user should also succeed");
}

// ── Jurisdiction enum mapping ──

#[tokio::test]
async fn jurisdiction_stored_as_enum() {
    let (db, user_id) = setup().await;
    let pool = db.pool();

    let created = ProjectRepository::create(pool, "enum-test", Jurisdiction::DE, user_id)
        .await
        .expect("create should succeed");

    let fetched = ProjectRepository::get_by_id(pool, created.id)
        .await
        .expect("get_by_id should succeed")
        .expect("project should exist");

    assert_eq!(
        fetched.jurisdiction,
        Jurisdiction::DE,
        "jurisdiction should roundtrip as DE"
    );
}

// ── Audit log ──

#[tokio::test]
async fn audit_log_written_on_create() {
    let (db, user_id) = setup().await;
    let pool = db.pool();

    let created = ProjectRepository::create(pool, "audit-create", Jurisdiction::NL, user_id)
        .await
        .expect("create should succeed");

    let log: Vec<(String,)> =
        sqlx::query_as("SELECT action FROM audit_log WHERE project_id = $1 ORDER BY timestamp")
            .bind(created.id)
            .fetch_all(pool)
            .await
            .expect("audit query should succeed");

    let actions: Vec<&str> = log.iter().map(|r| r.0.as_str()).collect();
    assert!(
        actions.contains(&"project.created"),
        "audit log should contain 'project.created', got: {actions:?}"
    );
}

#[tokio::test]
async fn audit_log_written_on_delete() {
    let (db, user_id) = setup().await;
    let pool = db.pool();

    let created = ProjectRepository::create(pool, "audit-delete", Jurisdiction::ES, user_id)
        .await
        .expect("create should succeed");

    ProjectRepository::delete(pool, created.id, user_id)
        .await
        .expect("delete should succeed");

    let log: Vec<(String,)> =
        sqlx::query_as("SELECT action FROM audit_log WHERE project_id = $1 ORDER BY timestamp")
            .bind(created.id)
            .fetch_all(pool)
            .await
            .expect("audit query should succeed");

    let actions: Vec<&str> = log.iter().map(|r| r.0.as_str()).collect();
    assert!(
        actions.contains(&"project.deleted"),
        "audit log should contain 'project.deleted', got: {actions:?}"
    );
}
