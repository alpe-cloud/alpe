//! Role-based access control (RBAC) policy for the Alpe platform.
//!
//! Provides a pure, deterministic policy function [`can`](crate::rbac::can) that evaluates whether
//! a given [`Role`](crate::rbac::Role) has a specific [`Permission`](crate::rbac::Permission). No IO, no side effects — the
//! entire RBAC matrix lives in code and is fully unit-testable.
//!
//! # Design
//!
//! Roles are hierarchical by convention but the policy is implemented as an
//! explicit match matrix (not bitmask) for auditability:
//!
//! | Role    | View | Create | Delete | Manage Members | Delete Project |
//! |---------|------|--------|--------|----------------|----------------|
//! | Owner   | ✓    | ✓      | ✓      | ✓              | ✓              |
//! | Admin   | ✓    | ✓      | ✓      | ✓              | ✗              |
//! | Member  | ✓    | ✓      | ✗      | ✗              | ✗              |
//! | Viewer  | ✓    | ✗      | ✗      | ✗              | ✗              |
//!
//! # Examples
//!
//! ```
//! use alpe_auth::rbac::{Role, Permission, can};
//!
//! assert!(can(Role::Owner, Permission::DeleteProject));
//! assert!(!can(Role::Viewer, Permission::CreateResource));
//! ```

use serde::{Deserialize, Serialize};

/// A user's role within a project.
///
/// Roles determine what actions a user may perform. The hierarchy
/// (from most to least privileged) is: `Owner` > `Admin` > `Member` > `Viewer`.
///
/// # Examples
///
/// ```
/// use alpe_auth::rbac::Role;
///
/// let role = Role::Admin;
/// assert_eq!(role.to_string(), "Admin");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display)]
pub enum Role {
    /// Full control including project deletion and ownership transfer.
    Owner,
    /// Manage resources and members, but cannot delete the project.
    Admin,
    /// Create and view resources, but cannot delete or manage members.
    Member,
    /// Read-only access to resources.
    Viewer,
}

/// An action that can be performed within a project.
///
/// Permissions are checked against a user's [`Role`] via the [`can`] function.
///
/// # Examples
///
/// ```
/// use alpe_auth::rbac::Permission;
///
/// let perm = Permission::ViewResource;
/// assert_eq!(perm.to_string(), "ViewResource");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display)]
pub enum Permission {
    /// View resources and their metadata.
    ViewResource,
    /// Create new resources within a project.
    CreateResource,
    /// Delete existing resources.
    DeleteResource,
    /// Manage project membership (invite, remove, change roles).
    ManageMembers,
    /// Permanently delete the project itself.
    DeleteProject,
}

/// Evaluates whether a [`Role`] is authorised to perform a [`Permission`].
///
/// This is a pure function with no side effects — the entire policy is an
/// explicit match matrix for auditability and testability.
///
/// # Examples
///
/// ```
/// use alpe_auth::rbac::{Role, Permission, can};
///
/// // Owners can do everything
/// assert!(can(Role::Owner, Permission::DeleteProject));
///
/// // Viewers can only view
/// assert!(can(Role::Viewer, Permission::ViewResource));
/// assert!(!can(Role::Viewer, Permission::CreateResource));
/// ```
#[must_use]
#[allow(clippy::match_same_arms)] // Intentionally explicit per-role arms for audit clarity
pub const fn can(role: Role, permission: Permission) -> bool {
    match (role, permission) {
        // Owner — full control
        (Role::Owner, _) => true,

        // Admin — everything except deleting the project
        (Role::Admin, Permission::DeleteProject) => false,
        (Role::Admin, _) => true,

        // Member — create and view resources only
        (Role::Member, Permission::ViewResource | Permission::CreateResource) => true,
        (Role::Member, _) => false,

        // Viewer — read-only
        (Role::Viewer, Permission::ViewResource) => true,
        (Role::Viewer, _) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: all permissions in the system.
    const ALL_PERMISSIONS: [Permission; 5] = [
        Permission::ViewResource,
        Permission::CreateResource,
        Permission::DeleteResource,
        Permission::ManageMembers,
        Permission::DeleteProject,
    ];

    #[test]
    fn owner_can_do_everything() {
        for perm in ALL_PERMISSIONS {
            assert!(can(Role::Owner, perm), "Owner should have {perm}");
        }
    }

    #[test]
    fn admin_can_manage_resources_and_members() {
        assert!(can(Role::Admin, Permission::CreateResource));
        assert!(can(Role::Admin, Permission::DeleteResource));
        assert!(can(Role::Admin, Permission::ViewResource));
        assert!(can(Role::Admin, Permission::ManageMembers));
    }

    #[test]
    fn admin_cannot_delete_project() {
        assert!(!can(Role::Admin, Permission::DeleteProject));
    }

    #[test]
    fn member_can_create_and_view_resources() {
        assert!(can(Role::Member, Permission::CreateResource));
        assert!(can(Role::Member, Permission::ViewResource));
    }

    #[test]
    fn member_cannot_delete_resources() {
        assert!(!can(Role::Member, Permission::DeleteResource));
    }

    #[test]
    fn member_cannot_manage_members() {
        assert!(!can(Role::Member, Permission::ManageMembers));
    }

    #[test]
    fn viewer_can_only_view() {
        assert!(can(Role::Viewer, Permission::ViewResource));
        for perm in ALL_PERMISSIONS {
            if perm != Permission::ViewResource {
                assert!(!can(Role::Viewer, perm), "Viewer should NOT have {perm}");
            }
        }
    }

    #[test]
    fn viewer_cannot_create_anything() {
        assert!(!can(Role::Viewer, Permission::CreateResource));
    }
}
