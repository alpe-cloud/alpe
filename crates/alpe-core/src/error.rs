//! Unified error hierarchy for the Alpe platform.
//!
//! Provides a layered error structure:
//! - [`CoreError`] — top-level enum bridging all domain errors
//! - [`ValidationError`] — input validation failures with field context
//! - [`TransitionError`] — invalid state machine transitions
//! - [`SovereigntyError`] — jurisdiction replication violations

/// Top-level error type for the `alpe-core` crate.
///
/// Aggregates all domain-specific error variants and provides `From`
/// conversions for ergonomic error propagation with `?`.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
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
/// Fields are private to enforce construction through [`ValidationError::new`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("validation error on '{field}': {message}")]
pub struct ValidationError {
    field: String,
    message: String,
}

impl ValidationError {
    /// Creates a new validation error for the given field.
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Returns the name of the field that failed validation.
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Returns the human-readable description of the validation failure.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Error returned when an invalid state machine transition is attempted.
///
/// Contains the source state and the event that was rejected.
/// Fields are private to enforce construction through [`TransitionError::new`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid transition: cannot apply '{event}' in state '{from}'")]
pub struct TransitionError {
    from: String,
    event: String,
}

impl TransitionError {
    /// Creates a new transition error for the given state and event.
    pub fn new(from: impl Into<String>, event: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            event: event.into(),
        }
    }

    /// Returns the state the resource was in when the invalid transition was attempted.
    pub fn from_state(&self) -> &str {
        &self.from
    }

    /// Returns the event that was rejected.
    pub fn event(&self) -> &str {
        &self.event
    }
}

/// Error returned when a sovereignty or jurisdiction constraint is violated.
///
/// Fields are private to enforce construction through [`SovereigntyError::new`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("sovereignty violation: {message}")]
pub struct SovereigntyError {
    message: String,
}

impl SovereigntyError {
    /// Creates a new sovereignty error with the given description.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the human-readable description of the sovereignty violation.
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_display_is_actionable() {
        let err = ValidationError::new("project_name", "must not be empty");
        let msg = err.to_string();
        assert!(msg.contains("project_name"), "should mention the field");
        assert!(
            msg.contains("must not be empty"),
            "should mention the reason"
        );
    }

    #[test]
    fn validation_error_accessors() {
        let err = ValidationError::new("email", "invalid format");
        assert_eq!(err.field(), "email");
        assert_eq!(err.message(), "invalid format");
    }

    #[test]
    fn transition_error_display_contains_state_and_event() {
        let err = TransitionError::new("Running", "ProvisionStarted");
        let msg = err.to_string();
        assert!(msg.contains("Running"));
        assert!(msg.contains("ProvisionStarted"));
    }

    #[test]
    fn transition_error_accessors() {
        let err = TransitionError::new("Pending", "Deploy");
        assert_eq!(err.from_state(), "Pending");
        assert_eq!(err.event(), "Deploy");
    }

    #[test]
    fn sovereignty_error_display_is_descriptive() {
        let err = SovereigntyError::new("DE cannot replicate to FR");
        assert!(err.to_string().contains("DE cannot replicate to FR"));
    }

    #[test]
    fn sovereignty_error_accessor() {
        let err = SovereigntyError::new("cross-country replication");
        assert_eq!(err.message(), "cross-country replication");
    }

    #[test]
    fn core_error_from_validation() {
        let val_err = ValidationError::new("name", "too short");
        let core_err: CoreError = val_err.into();
        assert!(matches!(core_err, CoreError::Validation(_)));
    }

    #[test]
    fn core_error_from_transition() {
        let trans_err = TransitionError::new("Pending", "UpdateCompleted");
        let core_err: CoreError = trans_err.into();
        assert!(matches!(core_err, CoreError::Transition(_)));
    }

    #[test]
    fn core_error_from_sovereignty() {
        let sov_err = SovereigntyError::new("cross-country replication");
        let core_err: CoreError = sov_err.into();
        assert!(matches!(core_err, CoreError::Sovereignty(_)));
    }

    #[test]
    fn core_error_is_clone() {
        let err = CoreError::from(ValidationError::new("x", "y"));
        let cloned = err.clone();
        // Verify the clone produces the same Display output
        assert_eq!(err.to_string(), cloned.to_string());
    }
}
