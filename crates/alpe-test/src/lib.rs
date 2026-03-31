//! # alpe-test
//!
//! Shared test infrastructure for the Alpe platform.
//!
//! This crate is a **dev-dependency only** — it is never compiled into any
//! shipped binary. It provides reusable test utilities used across all crates:
//!
//! - `TestDb` — spins up an ephemeral Postgres via testcontainers, runs migrations
//! - `TestApi` — boots the axum server on a random port with a `TestDb`
//! - `TestUser` — factory for creating test users with specific roles
//! - `Fixture` — trait for seeding test data
//! - Assert helpers for API response validation
