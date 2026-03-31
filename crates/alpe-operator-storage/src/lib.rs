//! # alpe-operator-storage
//!
//! Kubernetes operator for managed object storage resources on the Alpe platform.
//!
//! This operator watches for `ObjectStore` custom resources and reconciles them
//! by provisioning S3-compatible storage buckets. Sovereignty constraints ensure
//! storage is created in the jurisdiction specified by the project.
