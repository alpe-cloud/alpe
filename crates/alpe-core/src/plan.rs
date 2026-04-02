//! Compute and database service plans with resource limits.
//!
//! Defines the available service tiers for compute and database resources,
//! along with the resource limits (CPU, RAM, storage, connections) for each tier.
//!
//! # Examples
//!
//! ```
//! use alpe_core::plan::{ComputePlan, DatabasePlan};
//!
//! let limits = ComputePlan::Starter.limits();
//! assert_eq!(limits.cpu_millicores, 250);
//!
//! let db_limits = DatabasePlan::Starter.limits();
//! assert_eq!(db_limits.storage_gb, Some(5));
//! ```

/// Resource limits for a service plan.
///
/// Not all fields apply to every plan type: compute plans have no storage
/// or connection limits, and database plans always have storage and connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlanLimits {
    /// CPU allocation in millicores (1000 = 1 vCPU).
    pub cpu_millicores: u32,
    /// Memory allocation in megabytes.
    pub memory_mb: u32,
    /// Storage allocation in gigabytes (only for database plans).
    pub storage_gb: Option<u32>,
    /// Maximum number of connections (only for database plans).
    pub max_connections: Option<u32>,
    /// Whether the plan includes a read replica (only for database plans).
    pub read_replica: bool,
}

/// Compute service plans.
///
/// Plans scale monotonically: each tier provides at least as many resources
/// as the previous one.
///
/// # Serialization
///
/// Plans serialize to/from lowercase strings (e.g. `"starter"`, `"small"`).
///
/// # Examples
///
/// ```
/// use alpe_core::plan::ComputePlan;
/// use std::str::FromStr;
///
/// let plan = ComputePlan::from_str("starter").unwrap();
/// assert_eq!(plan.limits().cpu_millicores, 250);
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ComputePlan {
    /// Shared CPU (250m), 256 MB RAM.
    Starter,
    /// 0.5 vCPU (500m), 512 MB RAM.
    Small,
    /// 1 vCPU (1000m), 2048 MB RAM.
    Medium,
    /// 2 vCPU (2000m), 4096 MB RAM.
    Large,
    /// 4 vCPU (4000m), 8192 MB RAM.
    Xl,
}

impl ComputePlan {
    /// Returns the resource limits for this compute plan.
    ///
    /// Compute plans never have storage, connections, or read replicas.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpe_core::plan::ComputePlan;
    ///
    /// let limits = ComputePlan::Xl.limits();
    /// assert_eq!(limits.cpu_millicores, 4000);
    /// assert_eq!(limits.memory_mb, 8192);
    /// assert!(limits.storage_gb.is_none());
    /// ```
    #[must_use]
    pub const fn limits(&self) -> PlanLimits {
        match self {
            Self::Starter => PlanLimits {
                cpu_millicores: 250,
                memory_mb: 256,
                storage_gb: None,
                max_connections: None,
                read_replica: false,
            },
            Self::Small => PlanLimits {
                cpu_millicores: 500,
                memory_mb: 512,
                storage_gb: None,
                max_connections: None,
                read_replica: false,
            },
            Self::Medium => PlanLimits {
                cpu_millicores: 1000,
                memory_mb: 2048,
                storage_gb: None,
                max_connections: None,
                read_replica: false,
            },
            Self::Large => PlanLimits {
                cpu_millicores: 2000,
                memory_mb: 4096,
                storage_gb: None,
                max_connections: None,
                read_replica: false,
            },
            Self::Xl => PlanLimits {
                cpu_millicores: 4000,
                memory_mb: 8192,
                storage_gb: None,
                max_connections: None,
                read_replica: false,
            },
        }
    }
}

/// Database service plans.
///
/// Plans scale monotonically: each tier provides at least as many resources
/// as the previous one. Medium and above include read replicas.
///
/// # Examples
///
/// ```
/// use alpe_core::plan::DatabasePlan;
/// use std::str::FromStr;
///
/// let plan = DatabasePlan::from_str("medium").unwrap();
/// assert!(plan.limits().read_replica);
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DatabasePlan {
    /// 5 GB storage, 20 connections, no read replica.
    Starter,
    /// 10 GB storage, 50 connections, no read replica.
    Small,
    /// 25 GB storage, 100 connections, with read replica.
    Medium,
    /// 50 GB storage, 200 connections, with read replica.
    Large,
    /// 100 GB storage, 500 connections, with read replica.
    Xl,
}

impl DatabasePlan {
    /// Returns the resource limits for this database plan.
    ///
    /// Database plans always include storage and connection limits.
    /// Medium and above include a read replica.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpe_core::plan::DatabasePlan;
    ///
    /// let limits = DatabasePlan::Starter.limits();
    /// assert_eq!(limits.storage_gb, Some(5));
    /// assert_eq!(limits.max_connections, Some(20));
    /// assert!(!limits.read_replica);
    /// ```
    #[must_use]
    pub const fn limits(&self) -> PlanLimits {
        match self {
            Self::Starter => PlanLimits {
                cpu_millicores: 250,
                memory_mb: 512,
                storage_gb: Some(5),
                max_connections: Some(20),
                read_replica: false,
            },
            Self::Small => PlanLimits {
                cpu_millicores: 500,
                memory_mb: 1024,
                storage_gb: Some(10),
                max_connections: Some(50),
                read_replica: false,
            },
            Self::Medium => PlanLimits {
                cpu_millicores: 1000,
                memory_mb: 2048,
                storage_gb: Some(25),
                max_connections: Some(100),
                read_replica: true,
            },
            Self::Large => PlanLimits {
                cpu_millicores: 2000,
                memory_mb: 4096,
                storage_gb: Some(50),
                max_connections: Some(200),
                read_replica: true,
            },
            Self::Xl => PlanLimits {
                cpu_millicores: 4000,
                memory_mb: 8192,
                storage_gb: Some(100),
                max_connections: Some(500),
                read_replica: true,
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::str::FromStr;

    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn compute_starter_has_shared_cpu() {
        let limits = ComputePlan::Starter.limits();
        assert_eq!(limits.cpu_millicores, 250);
        assert_eq!(limits.memory_mb, 256);
    }

    #[test]
    fn compute_xl_has_4_vcpu() {
        let limits = ComputePlan::Xl.limits();
        assert_eq!(limits.cpu_millicores, 4000);
        assert_eq!(limits.memory_mb, 8192);
    }

    #[test]
    fn compute_plans_have_no_storage_or_connections() {
        for plan in ComputePlan::iter() {
            let limits = plan.limits();
            assert_eq!(limits.storage_gb, None, "{plan} should have no storage");
            assert_eq!(
                limits.max_connections, None,
                "{plan} should have no connections"
            );
        }
    }

    #[test]
    fn compute_plans_scale_monotonically() {
        let plans: Vec<ComputePlan> = ComputePlan::iter().collect();
        for window in plans.windows(2) {
            let (prev, next) = (window[0].limits(), window[1].limits());
            assert!(
                next.cpu_millicores >= prev.cpu_millicores,
                "CPU should scale monotonically"
            );
            assert!(
                next.memory_mb >= prev.memory_mb,
                "RAM should scale monotonically"
            );
        }
    }

    #[test]
    fn database_starter_has_5gb_storage() {
        let limits = DatabasePlan::Starter.limits();
        assert_eq!(limits.storage_gb, Some(5));
        assert_eq!(limits.max_connections, Some(20));
    }

    #[test]
    fn database_medium_and_above_have_read_replica() {
        assert!(!DatabasePlan::Starter.limits().read_replica);
        assert!(!DatabasePlan::Small.limits().read_replica);
        assert!(DatabasePlan::Medium.limits().read_replica);
        assert!(DatabasePlan::Large.limits().read_replica);
        assert!(DatabasePlan::Xl.limits().read_replica);
    }

    #[test]
    fn database_plans_scale_monotonically() {
        let plans: Vec<DatabasePlan> = DatabasePlan::iter().collect();
        for window in plans.windows(2) {
            let (prev, next) = (window[0].limits(), window[1].limits());
            assert!(
                next.storage_gb >= prev.storage_gb,
                "storage should scale monotonically"
            );
            assert!(
                next.max_connections >= prev.max_connections,
                "connections should scale monotonically"
            );
        }
    }

    #[test]
    fn compute_plan_parses_from_string() {
        let plan = ComputePlan::from_str("starter").expect("should parse 'starter'");
        assert_eq!(plan, ComputePlan::Starter);
    }

    #[test]
    fn plan_rejects_invalid_string() {
        assert!(ComputePlan::from_str("huge").is_err());
        assert!(DatabasePlan::from_str("huge").is_err());
    }

    #[test]
    fn compute_plan_displays_as_lowercase() {
        assert_eq!(ComputePlan::Starter.to_string(), "starter");
    }

    #[test]
    fn database_plan_roundtrips_through_display_and_parse() {
        for plan in DatabasePlan::iter() {
            let s = plan.to_string();
            let roundtrip = DatabasePlan::from_str(&s).expect("should roundtrip");
            assert_eq!(plan, roundtrip);
        }
    }
}
