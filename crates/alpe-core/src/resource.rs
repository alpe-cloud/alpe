//! Resource state machine and metadata.
//!
//! Resources in the Alpe platform follow a well-defined lifecycle managed
//! by a state machine. This module defines the states, events, and transition
//! rules that govern resource lifecycle management.
//!
//! # State Machine
//!
//! ```text
//!                    ┌──────────────┐
//!                    │   Pending    │
//!                    └──────┬───────┘
//!              ProvisionStarted│    │DeleteRequested
//!                    ┌────────▼┐   │
//!                    │Provision│   │
//!                    │  -ing   │   │
//!                    └──┬───┬──┘   │
//!     ProvisionCompleted│   │OperationFailed
//!                  ┌────▼┐  │      │
//!           ┌─────►│Run- │  │      │
//!           │      │ning │  │      │
//!           │      └─┬─┬─┘  │      │
//!   UpdateCompleted│  │ │    │      │
//!           │  Update│ │Delete     │
//!           │Requested │Requested  │
//!           │      ┌─▼─┐   │      │
//!           └──────│Upd-│   │      │
//!                  │ating│  │      │
//!                  └──┬──┘  │      │
//!        OperationFailed│   │      │
//!                  ┌──▼───┐ │      │
//!     RetryRequested│Failed├─┘      │
//!          ┌───────►└──┬───┘Delete  │
//!          │           │Requested   │
//!          │     ┌─────▼──────┐     │
//!          │     │  Deleting  │◄────┘
//!          │     └─────┬──────┘
//!          │  DeleteCompleted│
//!          │     ┌─────▼──────┐
//!          │     │  Deleted   │ (terminal)
//!          │     └────────────┘
//!          │
//!          └── (back to Pending)
//! ```
//!
//! # Examples
//!
//! ```
//! use alpe_core::resource::{ResourceState, ResourceEvent};
//!
//! let state = ResourceState::Pending;
//! let state = state.transition(ResourceEvent::ProvisionStarted).unwrap();
//! assert_eq!(state, ResourceState::Provisioning);
//! ```

use crate::error::TransitionError;

/// The lifecycle states a resource can be in.
///
/// Resources progress through these states via events. Only valid
/// transitions are accepted; invalid ones return a [`TransitionError`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display,
)]
pub enum ResourceState {
    /// Resource has been requested but provisioning has not started.
    Pending,
    /// Resource is being provisioned.
    Provisioning,
    /// Resource is live and accepting mutations.
    Running,
    /// Resource is being updated (e.g. plan change).
    Updating,
    /// Resource is being deleted.
    Deleting,
    /// Resource has been permanently deleted (terminal state).
    Deleted,
    /// A previous operation failed; retry or delete.
    Failed,
}

/// Events that drive resource state transitions.
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
    strum::EnumIter,
)]
pub enum ResourceEvent {
    /// Provisioning has been initiated.
    ProvisionStarted,
    /// Provisioning completed successfully.
    ProvisionCompleted,
    /// An update (e.g. plan change) was requested.
    UpdateRequested,
    /// An update completed successfully.
    UpdateCompleted,
    /// Resource deletion was requested.
    DeleteRequested,
    /// Resource deletion completed successfully.
    DeleteCompleted,
    /// An operation (provision/update) failed.
    OperationFailed,
    /// Retry was requested after a failure.
    RetryRequested,
}

impl ResourceState {
    /// Attempts to transition from this state given an event.
    ///
    /// Returns the new state on success, or a [`TransitionError`] if the
    /// event is not valid in the current state.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError`] when the event is not a valid transition
    /// from the current state. The error includes both the current state
    /// and the rejected event for debugging.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpe_core::resource::{ResourceState, ResourceEvent};
    ///
    /// let next = ResourceState::Pending
    ///     .transition(ResourceEvent::ProvisionStarted)
    ///     .unwrap();
    /// assert_eq!(next, ResourceState::Provisioning);
    ///
    /// // Invalid transition returns an error
    /// let err = ResourceState::Pending
    ///     .transition(ResourceEvent::UpdateCompleted);
    /// assert!(err.is_err());
    /// ```
    pub fn transition(self, event: ResourceEvent) -> Result<Self, TransitionError> {
        match (self, event) {
            // Pending
            (Self::Pending, ResourceEvent::ProvisionStarted) => Ok(Self::Provisioning),

            // Provisioning / Updating → Running
            (Self::Provisioning, ResourceEvent::ProvisionCompleted)
            | (Self::Updating, ResourceEvent::UpdateCompleted) => Ok(Self::Running),

            // Provisioning / Updating → Failed
            (Self::Provisioning | Self::Updating, ResourceEvent::OperationFailed) => {
                Ok(Self::Failed)
            }

            // Running
            (Self::Running, ResourceEvent::UpdateRequested) => Ok(Self::Updating),

            // Pending / Running / Failed → Deleting
            (Self::Pending | Self::Running | Self::Failed, ResourceEvent::DeleteRequested) => {
                Ok(Self::Deleting)
            }

            // Deleting → Deleted (terminal)
            (Self::Deleting, ResourceEvent::DeleteCompleted) => Ok(Self::Deleted),

            // Failed → retry
            (Self::Failed, ResourceEvent::RetryRequested) => Ok(Self::Pending),

            // Everything else is invalid
            _ => Err(TransitionError::new(self.to_string(), event.to_string())),
        }
    }

    /// Returns `true` if the resource in this state can accept mutation
    /// operations (updates).
    ///
    /// Only resources in the [`Running`](ResourceState::Running) state
    /// accept mutations.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpe_core::resource::ResourceState;
    ///
    /// assert!(ResourceState::Running.accepts_mutations());
    /// assert!(!ResourceState::Pending.accepts_mutations());
    /// ```
    #[must_use]
    pub const fn accepts_mutations(&self) -> bool {
        matches!(self, Self::Running)
    }
}

/// Metadata common to all resources.
///
/// Contains the unique identifier and timestamps for creation and last update.
///
/// # Examples
///
/// ```
/// use alpe_core::resource::ResourceMetadata;
///
/// let meta = ResourceMetadata::new();
/// assert!(!meta.id().is_nil());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResourceMetadata {
    id: uuid::Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl ResourceMetadata {
    /// Creates new metadata with a random UUID and the current timestamp.
    #[must_use]
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns the unique resource identifier.
    #[must_use]
    pub const fn id(&self) -> uuid::Uuid {
        self.id
    }

    /// Returns the creation timestamp.
    #[must_use]
    pub const fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }

    /// Returns the last-updated timestamp.
    #[must_use]
    pub const fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.updated_at
    }
}

impl Default for ResourceMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    // ── ResourceMetadata ──

    #[test]
    fn metadata_new_generates_uuid_and_timestamps() {
        let meta = ResourceMetadata::new();
        assert!(!meta.id().is_nil());
        assert_eq!(meta.created_at(), meta.updated_at());
    }

    #[test]
    fn metadata_fields_are_consistent() {
        let meta = ResourceMetadata::new();
        // Verify all accessors return consistent values
        assert!(!meta.id().is_nil());
        assert_eq!(meta.created_at(), meta.updated_at());
        // Clone produces equal metadata
        let cloned = meta.clone();
        assert_eq!(meta, cloned);
    }

    // ── Valid transitions ──

    #[test]
    fn pending_to_provisioning_on_provision_started() {
        let next = ResourceState::Pending
            .transition(ResourceEvent::ProvisionStarted)
            .expect("should succeed");
        assert_eq!(next, ResourceState::Provisioning);
    }

    #[test]
    fn pending_to_deleting_on_delete_requested() {
        let next = ResourceState::Pending
            .transition(ResourceEvent::DeleteRequested)
            .expect("should succeed");
        assert_eq!(next, ResourceState::Deleting);
    }

    #[test]
    fn provisioning_to_running_on_provision_completed() {
        let next = ResourceState::Provisioning
            .transition(ResourceEvent::ProvisionCompleted)
            .expect("should succeed");
        assert_eq!(next, ResourceState::Running);
    }

    #[test]
    fn provisioning_to_failed_on_operation_failed() {
        let next = ResourceState::Provisioning
            .transition(ResourceEvent::OperationFailed)
            .expect("should succeed");
        assert_eq!(next, ResourceState::Failed);
    }

    #[test]
    fn running_to_updating_on_update_requested() {
        let next = ResourceState::Running
            .transition(ResourceEvent::UpdateRequested)
            .expect("should succeed");
        assert_eq!(next, ResourceState::Updating);
    }

    #[test]
    fn running_to_deleting_on_delete_requested() {
        let next = ResourceState::Running
            .transition(ResourceEvent::DeleteRequested)
            .expect("should succeed");
        assert_eq!(next, ResourceState::Deleting);
    }

    #[test]
    fn updating_to_running_on_update_completed() {
        let next = ResourceState::Updating
            .transition(ResourceEvent::UpdateCompleted)
            .expect("should succeed");
        assert_eq!(next, ResourceState::Running);
    }

    #[test]
    fn updating_to_failed_on_operation_failed() {
        let next = ResourceState::Updating
            .transition(ResourceEvent::OperationFailed)
            .expect("should succeed");
        assert_eq!(next, ResourceState::Failed);
    }

    #[test]
    fn deleting_to_deleted_on_delete_completed() {
        let next = ResourceState::Deleting
            .transition(ResourceEvent::DeleteCompleted)
            .expect("should succeed");
        assert_eq!(next, ResourceState::Deleted);
    }

    #[test]
    fn failed_to_pending_on_retry_requested() {
        let next = ResourceState::Failed
            .transition(ResourceEvent::RetryRequested)
            .expect("should succeed");
        assert_eq!(next, ResourceState::Pending);
    }

    #[test]
    fn failed_to_deleting_on_delete_requested() {
        let next = ResourceState::Failed
            .transition(ResourceEvent::DeleteRequested)
            .expect("should succeed");
        assert_eq!(next, ResourceState::Deleting);
    }

    // ── Multi-step lifecycles ──

    #[test]
    fn full_happy_lifecycle() {
        let state = ResourceState::Pending;
        let state = state
            .transition(ResourceEvent::ProvisionStarted)
            .expect("Pending → Provisioning");
        let state = state
            .transition(ResourceEvent::ProvisionCompleted)
            .expect("Provisioning → Running");
        let state = state
            .transition(ResourceEvent::DeleteRequested)
            .expect("Running → Deleting");
        let state = state
            .transition(ResourceEvent::DeleteCompleted)
            .expect("Deleting → Deleted");
        assert_eq!(state, ResourceState::Deleted);
    }

    #[test]
    fn update_cycle() {
        let state = ResourceState::Running;
        let state = state
            .transition(ResourceEvent::UpdateRequested)
            .expect("Running → Updating");
        let state = state
            .transition(ResourceEvent::UpdateCompleted)
            .expect("Updating → Running");
        assert_eq!(state, ResourceState::Running);
    }

    #[test]
    fn retry_after_failure() {
        let state = ResourceState::Failed;
        let state = state
            .transition(ResourceEvent::RetryRequested)
            .expect("Failed → Pending");
        let state = state
            .transition(ResourceEvent::ProvisionStarted)
            .expect("Pending → Provisioning");
        let state = state
            .transition(ResourceEvent::ProvisionCompleted)
            .expect("Provisioning → Running");
        assert_eq!(state, ResourceState::Running);
    }

    // ── Invalid transitions ──

    #[test]
    fn pending_rejects_provision_completed() {
        assert!(
            ResourceState::Pending
                .transition(ResourceEvent::ProvisionCompleted)
                .is_err()
        );
    }

    #[test]
    fn pending_rejects_update_requested() {
        assert!(
            ResourceState::Pending
                .transition(ResourceEvent::UpdateRequested)
                .is_err()
        );
    }

    #[test]
    fn pending_rejects_update_completed() {
        assert!(
            ResourceState::Pending
                .transition(ResourceEvent::UpdateCompleted)
                .is_err()
        );
    }

    #[test]
    fn pending_rejects_operation_failed() {
        assert!(
            ResourceState::Pending
                .transition(ResourceEvent::OperationFailed)
                .is_err()
        );
    }

    #[test]
    fn pending_rejects_retry_requested() {
        assert!(
            ResourceState::Pending
                .transition(ResourceEvent::RetryRequested)
                .is_err()
        );
    }

    #[test]
    fn provisioning_rejects_update_requested() {
        assert!(
            ResourceState::Provisioning
                .transition(ResourceEvent::UpdateRequested)
                .is_err()
        );
    }

    #[test]
    fn provisioning_rejects_delete_requested() {
        assert!(
            ResourceState::Provisioning
                .transition(ResourceEvent::DeleteRequested)
                .is_err()
        );
    }

    #[test]
    fn running_rejects_provision_started() {
        assert!(
            ResourceState::Running
                .transition(ResourceEvent::ProvisionStarted)
                .is_err()
        );
    }

    #[test]
    fn running_rejects_provision_completed() {
        assert!(
            ResourceState::Running
                .transition(ResourceEvent::ProvisionCompleted)
                .is_err()
        );
    }

    #[test]
    fn running_rejects_retry_requested() {
        assert!(
            ResourceState::Running
                .transition(ResourceEvent::RetryRequested)
                .is_err()
        );
    }

    #[test]
    fn updating_rejects_provision_started() {
        assert!(
            ResourceState::Updating
                .transition(ResourceEvent::ProvisionStarted)
                .is_err()
        );
    }

    #[test]
    fn updating_rejects_delete_requested() {
        assert!(
            ResourceState::Updating
                .transition(ResourceEvent::DeleteRequested)
                .is_err()
        );
    }

    #[test]
    fn deleting_rejects_everything_except_delete_completed() {
        let non_delete_events =
            ResourceEvent::iter().filter(|e| *e != ResourceEvent::DeleteCompleted);

        for event in non_delete_events {
            assert!(
                ResourceState::Deleting.transition(event).is_err(),
                "Deleting should reject {event}"
            );
        }
    }

    #[test]
    fn deleted_rejects_all_events() {
        for event in ResourceEvent::iter() {
            assert!(
                ResourceState::Deleted.transition(event).is_err(),
                "Deleted should reject {event}"
            );
        }
    }

    #[test]
    fn failed_rejects_provision_started() {
        assert!(
            ResourceState::Failed
                .transition(ResourceEvent::ProvisionStarted)
                .is_err()
        );
    }

    #[test]
    fn failed_rejects_update_requested() {
        assert!(
            ResourceState::Failed
                .transition(ResourceEvent::UpdateRequested)
                .is_err()
        );
    }

    // ── Error context ──

    #[test]
    fn transition_error_contains_state_and_event() {
        let err = ResourceState::Running
            .transition(ResourceEvent::ProvisionStarted)
            .expect_err("should fail");
        assert_eq!(err.from_state(), "Running");
        assert_eq!(err.event(), "ProvisionStarted");
    }

    // ── Helpers ──

    #[test]
    fn only_running_accepts_mutations() {
        assert!(ResourceState::Running.accepts_mutations());
        assert!(!ResourceState::Pending.accepts_mutations());
        assert!(!ResourceState::Provisioning.accepts_mutations());
        assert!(!ResourceState::Updating.accepts_mutations());
        assert!(!ResourceState::Deleting.accepts_mutations());
        assert!(!ResourceState::Deleted.accepts_mutations());
        assert!(!ResourceState::Failed.accepts_mutations());
    }
}
