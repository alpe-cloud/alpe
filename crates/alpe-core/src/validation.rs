//! Input validation rules for the Alpe platform.
//!
//! Provides validators for user-supplied input such as project names,
//! resource names, and other identifiers. All validators follow DNS label
//! constraints (RFC 1123) for Kubernetes compatibility.
//!
//! # DNS Label Rules
//!
//! - Length: 2–63 characters
//! - Characters: lowercase alphanumeric and hyphens only
//! - Must start with a lowercase letter
//! - Must not end with a hyphen
//! - No consecutive hyphens
//!
//! # Examples
//!
//! ```
//! use alpe_core::validation::validate_name;
//!
//! assert!(validate_name("project_name", "my-app").is_ok());
//! assert!(validate_name("project_name", "BAD").is_err());
//! ```

use crate::error::ValidationError;

/// Validates that a name conforms to DNS label constraints (RFC 1123).
///
/// # Rules
///
/// - Must be between 2 and 63 characters long
/// - Must contain only lowercase ASCII letters, digits, and hyphens
/// - Must start with a lowercase ASCII letter
/// - Must not end with a hyphen
/// - Must not contain consecutive hyphens
///
/// # Errors
///
/// Returns a [`ValidationError`] with the given `field` name and a descriptive
/// message if the value violates any rule.
///
/// # Examples
///
/// ```
/// use alpe_core::validation::validate_name;
///
/// // Valid name
/// assert!(validate_name("project_name", "my-app").is_ok());
///
/// // Invalid: uppercase
/// let err = validate_name("project_name", "MyApp").unwrap_err();
/// assert_eq!(err.field(), "project_name");
/// ```
pub fn validate_name(field: &str, value: &str) -> Result<(), ValidationError> {
    validate_length(field, value)?;
    validate_boundaries(field, value)?;
    validate_chars(field, value)
}

/// Checks length constraints: non-empty, 2–63 characters.
fn validate_length(field: &str, value: &str) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::new(field, "must not be empty"));
    }
    if value.len() < 2 {
        return Err(ValidationError::new(field, "must be at least 2 characters"));
    }
    if value.len() > 63 {
        return Err(ValidationError::new(field, "must be at most 63 characters"));
    }
    Ok(())
}

/// Checks that the value starts with a lowercase letter and doesn't end with a hyphen.
fn validate_boundaries(field: &str, value: &str) -> Result<(), ValidationError> {
    let first = value.bytes().next().unwrap_or_default();
    if !first.is_ascii_lowercase() {
        return Err(ValidationError::new(
            field,
            "must start with a lowercase letter",
        ));
    }
    if value.ends_with('-') {
        return Err(ValidationError::new(field, "must not end with a hyphen"));
    }
    Ok(())
}

/// Checks that each character is lowercase alphanumeric or hyphen, with no consecutive hyphens.
fn validate_chars(field: &str, value: &str) -> Result<(), ValidationError> {
    let mut prev_was_hyphen = false;
    for ch in value.chars() {
        match ch {
            'a'..='z' | '0'..='9' => prev_was_hyphen = false,
            '-' => {
                if prev_was_hyphen {
                    return Err(ValidationError::new(
                        field,
                        "must not contain consecutive hyphens",
                    ));
                }
                prev_was_hyphen = true;
            }
            _ if ch.is_ascii_uppercase() => {
                return Err(ValidationError::new(
                    field,
                    "must contain only lowercase letters, digits, and hyphens",
                ));
            }
            _ => {
                return Err(ValidationError::new(
                    field,
                    format!("contains invalid character '{ch}'"),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_name() {
        assert!(validate_name("name", "my-app").is_ok());
    }

    #[test]
    fn accepts_minimum_length_name() {
        assert!(validate_name("name", "ab").is_ok());
    }

    #[test]
    fn accepts_maximum_length_name() {
        let name = "a".repeat(63);
        assert!(validate_name("name", &name).is_ok());
    }

    #[test]
    fn accepts_name_with_digits() {
        assert!(validate_name("name", "app-v2").is_ok());
    }

    #[test]
    fn rejects_empty_name() {
        let err = validate_name("name", "").expect_err("should fail");
        assert!(
            err.message().contains("empty"),
            "message should mention empty: {}",
            err.message()
        );
    }

    #[test]
    fn rejects_single_char_name() {
        let err = validate_name("name", "a").expect_err("should fail");
        assert!(
            err.message().contains("at least"),
            "message should mention 'at least': {}",
            err.message()
        );
    }

    #[test]
    fn rejects_too_long_name() {
        let name = "a".repeat(64);
        let err = validate_name("name", &name).expect_err("should fail");
        assert!(
            err.message().contains("at most"),
            "message should mention 'at most': {}",
            err.message()
        );
    }

    #[test]
    fn rejects_name_starting_with_digit() {
        let err = validate_name("name", "1app").expect_err("should fail");
        assert!(
            err.message().contains("start with"),
            "message should mention 'start with': {}",
            err.message()
        );
    }

    #[test]
    fn rejects_name_starting_with_hyphen() {
        let err = validate_name("name", "-app").expect_err("should fail");
        assert!(err.message().contains("start with"));
    }

    #[test]
    fn rejects_name_starting_with_uppercase() {
        let err = validate_name("name", "App").expect_err("should fail");
        assert!(err.message().contains("start with"));
    }

    #[test]
    fn rejects_name_ending_with_hyphen() {
        let err = validate_name("name", "app-").expect_err("should fail");
        assert!(err.message().contains("hyphen"));
    }

    #[test]
    fn rejects_consecutive_hyphens() {
        let err = validate_name("name", "my--app").expect_err("should fail");
        assert!(err.message().contains("consecutive"));
    }

    #[test]
    fn rejects_uppercase_letters() {
        let err = validate_name("name", "myApp").expect_err("should fail");
        assert!(err.message().contains("lowercase"));
    }

    #[test]
    fn rejects_underscores() {
        let err = validate_name("name", "my_app").expect_err("should fail");
        assert!(
            err.message().contains('_'),
            "message should mention the offending char: {}",
            err.message()
        );
    }

    #[test]
    fn rejects_spaces() {
        let err = validate_name("name", "my app").expect_err("should fail");
        assert!(!err.message().is_empty());
    }

    #[test]
    fn rejects_dots() {
        let err = validate_name("name", "my.app").expect_err("should fail");
        assert!(
            err.message().contains('.'),
            "message should mention the offending char: {}",
            err.message()
        );
    }

    #[test]
    fn error_carries_the_field_name() {
        let err = validate_name("project_name", "BAD").expect_err("should fail");
        assert_eq!(err.field(), "project_name");
    }
}
