//! # alpe-substrate
//!
//! Infrastructure abstraction layer for the Alpe platform.
//!
//! The substrate crate provides a uniform interface over the underlying
//! infrastructure providers (primarily Hetzner Cloud and bare-metal servers).
//! It abstracts server provisioning, network configuration, and storage
//! allocation so that the operator crates can remain provider-agnostic.
