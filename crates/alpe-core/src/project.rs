//! Project specification and validation.
//!
//! A project is the top-level organizational unit in the Alpe platform.
//! Every resource belongs to a project, and every project is bound to a
//! specific jurisdiction.
//!
//! # Examples
//!
//! ```
//! use alpe_core::project::ProjectSpec;
//! use alpe_core::jurisdiction::Jurisdiction;
//!
//! let spec = ProjectSpec::new("my-project", Jurisdiction::DE);
//! assert!(spec.validate().is_ok());
//! ```

use crate::error::ValidationError;
use crate::jurisdiction::Jurisdiction;
use crate::validation::validate_name;

/// Specification for creating a new project.
///
/// Contains the project name and jurisdiction. Use [`validate`](ProjectSpec::validate)
/// to check that all fields conform to the platform rules.
///
/// # Examples
///
/// ```
/// use alpe_core::project::ProjectSpec;
/// use alpe_core::jurisdiction::Jurisdiction;
///
/// let spec = ProjectSpec::new("my-project", Jurisdiction::DE);
/// assert!(spec.validate().is_ok());
/// assert_eq!(spec.name(), "my-project");
/// assert_eq!(spec.jurisdiction(), Jurisdiction::DE);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectSpec {
    name: String,
    jurisdiction: Jurisdiction,
}

impl ProjectSpec {
    /// Creates a new project specification.
    ///
    /// This does not validate the inputs; call [`validate`](ProjectSpec::validate)
    /// to check the name and jurisdiction rules.
    #[must_use]
    pub fn new(name: impl Into<String>, jurisdiction: Jurisdiction) -> Self {
        Self {
            name: name.into(),
            jurisdiction,
        }
    }

    /// Returns the project name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the project jurisdiction.
    #[must_use]
    pub const fn jurisdiction(&self) -> Jurisdiction {
        self.jurisdiction
    }

    /// Validates the project specification.
    ///
    /// Checks that the project name conforms to DNS label constraints
    /// (see [`validate_name`]).
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if the project name is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpe_core::project::ProjectSpec;
    /// use alpe_core::jurisdiction::Jurisdiction;
    ///
    /// // Valid spec
    /// let spec = ProjectSpec::new("my-project", Jurisdiction::DE);
    /// assert!(spec.validate().is_ok());
    ///
    /// // Invalid name
    /// let spec = ProjectSpec::new("BAD", Jurisdiction::DE);
    /// assert!(spec.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_name("name", &self.name)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn valid_project_spec_passes_validation() {
        let spec = ProjectSpec::new("my-project", Jurisdiction::DE);
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn project_rejects_invalid_name() {
        let spec = ProjectSpec::new("BAD", Jurisdiction::DE);
        assert!(spec.validate().is_err());
    }

    #[test]
    fn project_accessors_return_correct_values() {
        let spec = ProjectSpec::new("my-project", Jurisdiction::FR);
        assert_eq!(spec.name(), "my-project");
        assert_eq!(spec.jurisdiction(), Jurisdiction::FR);

        // Clone produces equal spec
        let cloned = spec.clone();
        assert_eq!(spec, cloned);
    }
}
