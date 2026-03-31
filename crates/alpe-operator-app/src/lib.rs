//! # alpe-operator-app
//!
//! Kubernetes operator for managed application deployments on the Alpe platform.
//!
//! This operator watches for `App` custom resources and reconciles them by
//! creating Kubernetes `Deployment`, `Service`, and `Ingress` resources.
//! It handles scaling, rolling updates, and health monitoring while respecting
//! the [`alpe_core`] jurisdiction constraints for workload placement.
