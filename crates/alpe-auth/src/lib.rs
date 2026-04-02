//! # alpe-auth
//!
//! JWT authentication, password hashing, and role-based access control (RBAC)
//! for the Alpe platform.
//!
//! This crate is intentionally pure — all auth logic can be tested without a
//! running server or database. The axum extractor integration is the only
//! framework-coupled piece.
//!
//! ## Modules
//!
//! - [`rbac`] — `Role`, `Permission`, and the [`can`](rbac::can) policy function
//! - [`jwt`] — Token issuance ([`issue_token`](jwt::issue_token)) and validation ([`validate_token`](jwt::validate_token))
//! - [`extractor`] — Axum [`FromRequestParts`](axum::extract::FromRequestParts) implementation for [`AuthenticatedUser`](extractor::AuthenticatedUser)
//!
//! ## Design Principles
//!
//! - **Pure logic**: RBAC and JWT modules are deterministic and side-effect-free
//! - **Test-first**: All modules developed with strict TDD (RED → GREEN → Refactor)
//! - **Minimal coupling**: Only the `extractor` module depends on axum
//! - **Documentation as code**: `#![deny(missing_docs)]` enforces docs on every public item

/// Axum extractor for JWT-authenticated requests.
pub mod extractor;

/// JWT token issuance and validation.
pub mod jwt;

/// Role-based access control policy.
pub mod rbac;
