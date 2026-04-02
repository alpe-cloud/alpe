//! # alpe-core
//!
//! Core domain types, error hierarchy, and pure business logic for the Alpe platform.
//!
//! This crate is the foundation of the Alpe architecture. It contains zero IO — all types
//! and functions are pure, deterministic, and 100% unit-testable. Every other crate in the
//! workspace depends on `alpe-core`.
//!
//! ## Modules
//!
//! - [`error`] — Unified error hierarchy ([`error::CoreError`], [`error::ValidationError`], [`error::TransitionError`])
//! - [`jurisdiction`] — EU member state types and sovereignty replication rules
//! - [`resource`] — Resource state machine and metadata
//! - [`validation`] — Input validation (DNS label constraints for names)
//! - [`plan`] — Compute and database service plans with resource limits
//! - [`project`] — Project specification and validation
//!
//! ## Design Principles
//!
//! - **Pure logic**: No IO, no async, no side effects — every function is deterministic
//! - **Type safety**: Invalid states are unrepresentable where possible
//! - **Test-first**: All modules are developed with strict TDD (RED → GREEN → Refactor)
//! - **Documentation as code**: `#![deny(missing_docs)]` enforces documentation on every public item

/// Unified error hierarchy for the Alpe platform.
pub mod error;

/// EU jurisdiction types and sovereignty-aware replication rules.
pub mod jurisdiction;

/// Service plans and resource limits (compute, database).
pub mod plan;

/// Project specification and validation.
pub mod project;

/// Resource state machine and metadata.
pub mod resource;

/// Input validation rules (DNS label constraints).
pub mod validation;
