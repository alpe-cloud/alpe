//! Unified error hierarchy for the Alpe platform.
//!
//! Provides a layered error structure:
//! - `CoreError` — top-level enum bridging all domain errors
//! - `ValidationError` — input validation failures with field context
//! - `TransitionError` — invalid state machine transitions
//! - `SovereigntyError` — jurisdiction replication violations

use std::fmt;

/// Top-level error type for the `alpe-core` crate.
///
/// Aggregates all domain-specific error variants and provides `From`
/// conversions for ergonomic error propagation with `?`.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// An input validation rule was violated.
    #[error(transparent)]
    Validation(#[from] ValidationError),

    /// An invalid state machine transition was attempted.
    #[error(transparent)]
    Transition(#[from] TransitionError),

    /// A sovereignty / jurisdiction constraint was violated.
    #[error(transparent)]
    Sovereignty(#[from] SovereigntyError),
}

/// Error returned when an input value fails validation.
///
/// Carries the field name and a human-readable message describing the violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// The name of the field that failed validation.
    pub field: String,
    /// A human-readable description of the validation failure.
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "validation error on '{}': {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Error returned when an invalid state machine transition is attempted.
///
/// Contains the source state and the event that was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionError {
    /// The state the resource was in when the invalid transition was attempted.
    pub from: String,
    /// The event that was rejected.
    pub event: String,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid transition: cannot apply '{}' in state '{}'",
            self.event, self.from
        )
    }
}

impl std::error::Error for TransitionError {}

/// Error returned when a sovereignty or jurisdiction constraint is violated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SovereigntyError {
    /// A human-readable description of the sovereignty violation.
    pub message: String,
}

impl fmt::Display for SovereigntyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sovereignty violation: {}", self.message)
    }
}

impl std::error::Error for SovereigntyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_display_is_actionable() {
        let err = ValidationError {
            field: "project_name".to_string(),
            message: "must not be empty".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("project_name"), "should mention the field");
        assert!(
            msg.contains("must not be empty"),
            "should mention the reason"
        );
    }

    #[test]
    fn transition_error_display_contains_state_and_event() {
        let err = TransitionError {
            from: "Running".to_string(),
            event: "ProvisionStarted".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Running"));
        assert!(msg.contains("ProvisionStarted"));
    }

    #[test]
    fn sovereignty_error_display_is_descriptive() {
        let err = SovereigntyError {
            message: "DE cannot replicate to FR".to_string(),
        };
        assert!(err.to_string().contains("DE cannot replicate to FR"));
    }

    #[test]
    fn core_error_from_validation() {
        let val_err = ValidationError {
            field: "name".to_string(),
            message: "too short".to_string(),
        };
        let core_err: CoreError = val_err.into();
        assert!(matches!(core_err, CoreError::Validation(_)));
    }

    #[test]
    fn core_error_from_transition() {
        let trans_err = TransitionError {
            from: "Pending".to_string(),
            event: "UpdateCompleted".to_string(),
        };
        let core_err: CoreError = trans_err.into();
        assert!(matches!(core_err, CoreError::Transition(_)));
    }

    #[test]
    fn core_error_from_sovereignty() {
        let sov_err = SovereigntyError {
            message: "cross-country replication".to_string(),
        };
        let core_err: CoreError = sov_err.into();
        assert!(matches!(core_err, CoreError::Sovereignty(_)));
    }
}
