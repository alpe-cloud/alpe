//! # alpe-api
//!
//! Axum-based HTTP API server for the Alpe platform.
//!
//! This crate provides the REST API that the CLI and SDK communicate with.
//! It handles authentication, project management, and resource lifecycle
//! operations. Handlers are intentionally thin — they parse requests, delegate
//! to [`alpe_core`] for business logic, and format responses.
//!
//! ## Architecture
//!
//! - **Handlers** live in route-specific modules and are thin request/response adapters
//! - **Middleware** provides cross-cutting concerns (auth, tracing, error mapping)
//! - **State** is injected via axum's `State` extractor (database pool, auth config)
//! - **`OpenAPI`** spec is generated via [`utoipa`] and served at `/openapi.json`
