//! # alpe-auth
//!
//! JWT authentication, password hashing, and role-based access control (RBAC)
//! for the Alpe platform.
//!
//! This crate is intentionally pure — all auth logic can be tested without a
//! running server or database. The axum extractor integration is the only
//! framework-coupled piece.
//!
//! ## Modules (planned)
//!
//! - **RBAC** — `Role`, `Permission`, and the `can(role, permission)` policy function
//! - **JWT** — Token issuance (`issue_token`) and validation (`validate_token`)
//! - **Password** — Argon2 hashing and verification
//! - **Extractor** — Axum `FromRequestParts` implementation for `AuthenticatedUser`
