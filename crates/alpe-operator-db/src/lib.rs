//! # alpe-operator-db
//!
//! Kubernetes operator for managed database resources on the Alpe platform.
//!
//! This operator watches for `Database` custom resources and reconciles them
//! by provisioning, configuring, and managing `PostgreSQL` instances on the
//! underlying infrastructure. It enforces jurisdiction constraints from
//! [`alpe_core`] to ensure databases are deployed in the correct EU region.
