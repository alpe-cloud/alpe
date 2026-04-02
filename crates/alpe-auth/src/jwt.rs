//! JWT token issuance and validation for the Alpe platform.
//!
//! Provides [`issue_token`](crate::jwt::issue_token) and [`validate_token`](crate::jwt::validate_token) functions that are pure (aside from
//! reading the system clock for `iat`). All crypto is handled by the `jsonwebtoken` crate
//! using HMAC-SHA256.
//!
//! # Examples
//!
//! ```
//! use alpe_auth::jwt::{JwtConfig, issue_token, validate_token};
//! use alpe_auth::rbac::Role;
//! use uuid::Uuid;
//! use std::time::Duration;
//!
//! let config = JwtConfig::builder("super-secret-key-that-is-long-enough-for-hs256")
//!     .ttl(Duration::from_secs(3600))
//!     .issuer("alpe-platform")
//!     .audience("alpe-api")
//!     .build()
//!     .expect("config should be valid");
//! let user_id = Uuid::new_v4();
//!
//! let token = issue_token(&config, user_id, "alice@example.com", &[Role::Admin])
//!     .expect("token issuance should succeed");
//!
//! let claims = validate_token(&config, &token)
//!     .expect("token validation should succeed");
//!
//! assert_eq!(claims.sub(), user_id);
//! assert_eq!(claims.email(), "alice@example.com");
//! ```

use std::fmt;
use std::time::Duration;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rbac::Role;

/// Minimum secret length in bytes for HMAC-SHA256 security.
const MIN_SECRET_LENGTH: usize = 32;

/// Default token time-to-live (1 hour).
const DEFAULT_TTL: Duration = Duration::from_secs(3600);

/// Default clock leeway for token validation (60 seconds).
///
/// Accounts for clock skew between Kubernetes pods in a distributed cluster.
const DEFAULT_LEEWAY: u64 = 60;

/// Builder for [`JwtConfig`].
///
/// # Examples
///
/// ```
/// use alpe_auth::jwt::JwtConfig;
/// use std::time::Duration;
///
/// let config = JwtConfig::builder("a-secret-key-that-is-long-enough-for-hmac")
///     .ttl(Duration::from_secs(1800))
///     .issuer("my-service")
///     .audience("my-api")
///     .leeway(30)
///     .build()
///     .expect("config should be valid");
/// ```
pub struct JwtConfigBuilder {
    secret: String,
    ttl: Duration,
    issuer: String,
    audience: String,
    leeway: u64,
}

impl JwtConfigBuilder {
    /// Sets the token time-to-live.
    #[must_use]
    pub const fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Sets the token issuer (`iss` claim).
    #[must_use]
    pub fn issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = issuer.into();
        self
    }

    /// Sets the expected audience (`aud` claim).
    #[must_use]
    pub fn audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = audience.into();
        self
    }

    /// Sets the clock leeway in seconds for token validation.
    ///
    /// Default is 60 seconds, which accommodates typical NTP drift in
    /// Kubernetes clusters. Set to 0 for strict validation in tests.
    #[must_use]
    pub const fn leeway(mut self, leeway: u64) -> Self {
        self.leeway = leeway;
        self
    }

    /// Builds the [`JwtConfig`], validating that the secret meets minimum
    /// length requirements.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::ConfigInvalid`] if the secret is shorter than
    /// 32 bytes (the minimum for HMAC-SHA256 security).
    pub fn build(self) -> Result<JwtConfig, AuthError> {
        if self.secret.len() < MIN_SECRET_LENGTH {
            return Err(AuthError::ConfigInvalid(format!(
                "JWT secret must be at least {MIN_SECRET_LENGTH} bytes, got {}",
                self.secret.len()
            )));
        }

        Ok(JwtConfig {
            secret: SecretString::from(self.secret),
            ttl: self.ttl,
            issuer: self.issuer,
            audience: self.audience,
            leeway: self.leeway,
        })
    }
}

/// Configuration for JWT token issuance and validation.
///
/// The secret is stored as a [`SecretString`] and zeroized on drop to prevent
/// credential leaks from crash dumps or `/proc/mem` reads. The `Debug`
/// implementation redacts the secret field.
///
/// Use [`JwtConfig::builder`] to create a new configuration.
///
/// # Examples
///
/// ```
/// use alpe_auth::jwt::JwtConfig;
/// use std::time::Duration;
///
/// let config = JwtConfig::builder("a-secret-key-that-is-long-enough-for-hmac")
///     .ttl(Duration::from_secs(3600))
///     .build()
///     .expect("config should be valid");
/// assert_eq!(config.ttl(), Duration::from_secs(3600));
///
/// // Debug output redacts the secret
/// let debug = format!("{config:?}");
/// assert!(debug.contains("[REDACTED]"));
/// assert!(!debug.contains("secret-key"));
/// ```
pub struct JwtConfig {
    secret: SecretString,
    ttl: Duration,
    issuer: String,
    audience: String,
    leeway: u64,
}

// Manual Debug implementation to redact the secret (#8)
impl fmt::Debug for JwtConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JwtConfig")
            .field("secret", &"[REDACTED]")
            .field("ttl", &self.ttl)
            .field("issuer", &self.issuer)
            .field("audience", &self.audience)
            .field("leeway", &self.leeway)
            .finish()
    }
}

impl Clone for JwtConfig {
    fn clone(&self) -> Self {
        Self {
            secret: SecretString::from(self.secret.expose_secret().to_string()),
            ttl: self.ttl,
            issuer: self.issuer.clone(),
            audience: self.audience.clone(),
            leeway: self.leeway,
        }
    }
}

impl JwtConfig {
    /// Creates a new builder for JWT configuration.
    ///
    /// # Arguments
    ///
    /// - `secret` — the HMAC-SHA256 signing key (must be ≥ 32 bytes)
    #[must_use]
    pub fn builder(secret: impl Into<String>) -> JwtConfigBuilder {
        JwtConfigBuilder {
            secret: secret.into(),
            ttl: DEFAULT_TTL,
            issuer: String::from("alpe"),
            audience: String::from("alpe"),
            leeway: DEFAULT_LEEWAY,
        }
    }

    /// Returns the configured token time-to-live.
    #[must_use]
    pub const fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Returns the configured issuer.
    #[must_use]
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    /// Returns the configured audience.
    #[must_use]
    pub fn audience(&self) -> &str {
        &self.audience
    }

    /// Returns the configured clock leeway in seconds.
    #[must_use]
    pub const fn leeway(&self) -> u64 {
        self.leeway
    }
}

/// JWT claims payload embedded in every issued token.
///
/// Fields are private; use the accessor methods to read claim values.
/// Includes standard JWT claims (`iss`, `aud`, `sub`, `exp`, `iat`) plus
/// custom claims (`email`, `roles`).
///
/// # Examples
///
/// ```
/// use alpe_auth::jwt::{JwtConfig, issue_token, validate_token};
/// use alpe_auth::rbac::Role;
/// use uuid::Uuid;
/// use std::time::Duration;
///
/// let config = JwtConfig::builder("a-secret-key-that-is-long-enough-for-hmac")
///     .ttl(Duration::from_secs(3600))
///     .build()
///     .unwrap();
/// let uid = Uuid::new_v4();
/// let token = issue_token(&config, uid, "bob@test.com", &[Role::Member]).unwrap();
/// let claims = validate_token(&config, &token).unwrap();
///
/// assert_eq!(claims.sub(), uid);
/// assert_eq!(claims.email(), "bob@test.com");
/// assert_eq!(claims.roles(), &[Role::Member]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Token issuer.
    iss: String,
    /// Intended audience.
    aud: String,
    /// Subject (user ID).
    sub: Uuid,
    /// User email address.
    email: String,
    /// User roles for RBAC.
    roles: Vec<Role>,
    /// Expiration time (seconds since epoch).
    exp: u64,
    /// Issued-at time (seconds since epoch).
    iat: u64,
}

impl Claims {
    /// Returns the token issuer (`iss` claim).
    #[must_use]
    pub fn issuer(&self) -> &str {
        &self.iss
    }

    /// Returns the token audience (`aud` claim).
    #[must_use]
    pub fn audience(&self) -> &str {
        &self.aud
    }

    /// Returns the user ID (`sub` claim).
    #[must_use]
    pub const fn sub(&self) -> Uuid {
        self.sub
    }

    /// Returns the user email.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    /// Returns the roles embedded in the token.
    #[must_use]
    pub fn roles(&self) -> &[Role] {
        &self.roles
    }
}

/// Errors that can occur during authentication operations.
///
/// # Examples
///
/// ```
/// use alpe_auth::jwt::AuthError;
///
/// let err = AuthError::TokenExpired;
/// assert_eq!(err.to_string(), "token has expired");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuthError {
    /// The token has passed its expiration time.
    #[error("token has expired")]
    TokenExpired,

    /// The token is malformed, has an invalid signature, or is otherwise invalid.
    #[error("token is invalid")]
    TokenInvalid,

    /// No token was provided in the request.
    #[error("authentication token is missing")]
    TokenMissing,

    /// The authenticated user lacks the required permission.
    #[error("insufficient permissions")]
    InsufficientPermission,

    /// The JWT configuration is invalid (e.g., secret too short).
    #[error("invalid JWT configuration: {0}")]
    ConfigInvalid(String),
}

/// Issues a signed JWT token for the given user.
///
/// The token is signed with HMAC-SHA256 and contains the user's ID, email,
/// roles, and standard claims (`iss`, `aud`, `exp`, `iat`). The expiration is
/// calculated from the current system time plus the configured TTL.
///
/// # Errors
///
/// Returns [`AuthError::TokenInvalid`] if token encoding fails (e.g., due to
/// an invalid secret key — should not happen in practice).
///
/// # Examples
///
/// ```
/// use alpe_auth::jwt::{JwtConfig, issue_token};
/// use alpe_auth::rbac::Role;
/// use uuid::Uuid;
/// use std::time::Duration;
///
/// let config = JwtConfig::builder("a-secret-key-that-is-long-enough-for-hmac")
///     .ttl(Duration::from_secs(3600))
///     .build()
///     .unwrap();
/// let token = issue_token(&config, Uuid::new_v4(), "alice@test.com", &[Role::Owner]);
/// assert!(token.is_ok());
/// ```
#[tracing::instrument(skip_all, fields(user_id = %user_id))]
pub fn issue_token(
    config: &JwtConfig,
    user_id: Uuid,
    email: &str,
    roles: &[Role],
) -> Result<String, AuthError> {
    #[allow(clippy::cast_sign_loss)] // Timestamps are always positive (post-1970)
    let now = chrono::Utc::now().timestamp() as u64;
    let exp = now.saturating_add(config.ttl.as_secs());

    let claims = Claims {
        iss: config.issuer.clone(),
        aud: config.audience.clone(),
        sub: user_id,
        email: email.to_owned(),
        roles: roles.to_vec(),
        exp,
        iat: now,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.expose_secret().as_bytes()),
    )
    .map_err(|_| AuthError::TokenInvalid)
}

/// Validates a JWT token and returns the embedded [`Claims`].
///
/// Checks the token signature (HMAC-SHA256), expiration, issuer (`iss`),
/// and audience (`aud`). Does **not** perform authorization — use
/// [`crate::rbac::can`] after validation.
///
/// Clock leeway is configurable via [`JwtConfig::builder`] to accommodate
/// NTP drift in distributed Kubernetes clusters (default: 60 seconds).
///
/// # Errors
///
/// - [`AuthError::TokenExpired`] — the token has passed its `exp` claim
/// - [`AuthError::TokenInvalid`] — the token is malformed, empty, signed with a different key,
///   or has mismatched `iss`/`aud` claims
///
/// # Examples
///
/// ```
/// use alpe_auth::jwt::{JwtConfig, issue_token, validate_token};
/// use alpe_auth::rbac::Role;
/// use uuid::Uuid;
/// use std::time::Duration;
///
/// let config = JwtConfig::builder("a-secret-key-that-is-long-enough-for-hmac")
///     .ttl(Duration::from_secs(3600))
///     .build()
///     .unwrap();
/// let token = issue_token(&config, Uuid::new_v4(), "a@b.com", &[Role::Viewer]).unwrap();
/// let claims = validate_token(&config, &token);
/// assert!(claims.is_ok());
/// ```
#[tracing::instrument(skip_all)]
pub fn validate_token(config: &JwtConfig, token: &str) -> Result<Claims, AuthError> {
    let mut validation = Validation::default();
    validation.set_required_spec_claims(&["exp", "sub", "iss", "aud"]);
    validation.leeway = config.leeway;
    validation.set_issuer(&[&config.issuer]);
    validation.set_audience(&[&config.audience]);

    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.expose_secret().as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|err| {
        tracing::warn!(error = %err, "JWT validation failed");
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::TokenInvalid,
        }
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    /// Secret long enough for HMAC-SHA256 (≥32 bytes).
    const TEST_SECRET: &str = "test-secret-key-that-is-long-enough-for-hmac";

    fn test_config(ttl_secs: u64) -> JwtConfig {
        JwtConfig::builder(TEST_SECRET)
            .ttl(Duration::from_secs(ttl_secs))
            .leeway(0) // Strict for tests — no clock skew tolerance
            .build()
            .expect("test config should be valid")
    }

    #[test]
    fn issue_and_validate_roundtrip() {
        let config = test_config(3600);
        let user_id = Uuid::new_v4();
        let email = "alice@example.com";
        let roles = vec![Role::Admin];

        let token = issue_token(&config, user_id, email, &roles).expect("should issue token");
        let claims = validate_token(&config, &token).expect("should validate token");

        assert_eq!(claims.sub(), user_id);
        assert_eq!(claims.email(), email);
        assert_eq!(claims.roles(), &roles);
    }

    #[test]
    fn token_contains_user_id_and_email() {
        let config = test_config(3600);
        let user_id = Uuid::new_v4();
        let email = "bob@example.com";

        let token = issue_token(&config, user_id, email, &[Role::Member]).expect("should issue");
        let claims = validate_token(&config, &token).expect("should validate");

        assert_eq!(claims.sub(), user_id);
        assert_eq!(claims.email(), email);
    }

    #[test]
    fn token_contains_roles() {
        let config = test_config(3600);
        let roles = vec![Role::Admin, Role::Member];

        let token = issue_token(&config, Uuid::new_v4(), "x@y.com", &roles).expect("should issue");
        let claims = validate_token(&config, &token).expect("should validate");

        assert_eq!(claims.roles(), &roles);
    }

    #[test]
    fn token_contains_iss_and_aud() {
        let config = JwtConfig::builder(TEST_SECRET)
            .ttl(Duration::from_secs(3600))
            .issuer("test-issuer")
            .audience("test-audience")
            .leeway(0)
            .build()
            .expect("config should be valid");

        let token =
            issue_token(&config, Uuid::new_v4(), "x@y.com", &[Role::Viewer]).expect("should issue");
        let claims = validate_token(&config, &token).expect("should validate");

        assert_eq!(claims.issuer(), "test-issuer");
        assert_eq!(claims.audience(), "test-audience");
    }

    #[test]
    fn token_with_wrong_issuer_is_rejected() {
        let config_a = JwtConfig::builder(TEST_SECRET)
            .issuer("service-a")
            .leeway(0)
            .build()
            .expect("config should be valid");
        let config_b = JwtConfig::builder(TEST_SECRET)
            .issuer("service-b")
            .leeway(0)
            .build()
            .expect("config should be valid");

        let token = issue_token(&config_a, Uuid::new_v4(), "x@y.com", &[Role::Owner])
            .expect("should issue");

        let result = validate_token(&config_b, &token);
        assert_eq!(result, Err(AuthError::TokenInvalid));
    }

    #[test]
    fn token_with_wrong_audience_is_rejected() {
        let config_a = JwtConfig::builder(TEST_SECRET)
            .audience("api-a")
            .leeway(0)
            .build()
            .expect("config should be valid");
        let config_b = JwtConfig::builder(TEST_SECRET)
            .audience("api-b")
            .leeway(0)
            .build()
            .expect("config should be valid");

        let token = issue_token(&config_a, Uuid::new_v4(), "x@y.com", &[Role::Owner])
            .expect("should issue");

        let result = validate_token(&config_b, &token);
        assert_eq!(result, Err(AuthError::TokenInvalid));
    }

    #[test]
    fn expired_token_is_rejected() {
        let config = test_config(0); // TTL = 0 seconds → exp == iat

        let token =
            issue_token(&config, Uuid::new_v4(), "x@y.com", &[Role::Viewer]).expect("should issue");

        // Sleep just past the current second so the token's exp is in the past.
        // With leeway=0, jsonwebtoken rejects tokens where now > exp.
        std::thread::sleep(Duration::from_millis(1100));

        let result = validate_token(&config, &token);
        assert_eq!(result, Err(AuthError::TokenExpired));
    }

    #[test]
    fn malformed_token_is_rejected() {
        let config = test_config(3600);
        let result = validate_token(&config, "not.a.jwt");
        assert_eq!(result, Err(AuthError::TokenInvalid));
    }

    #[test]
    fn token_with_wrong_secret_is_rejected() {
        let config_a = JwtConfig::builder("secret-a-that-is-long-enough-for-hmac-sha256")
            .leeway(0)
            .build()
            .expect("config should be valid");
        let config_b = JwtConfig::builder("secret-b-that-is-long-enough-for-hmac-sha256")
            .leeway(0)
            .build()
            .expect("config should be valid");

        let token = issue_token(&config_a, Uuid::new_v4(), "x@y.com", &[Role::Owner])
            .expect("should issue");

        let result = validate_token(&config_b, &token);
        assert_eq!(result, Err(AuthError::TokenInvalid));
    }

    #[test]
    fn empty_token_is_rejected() {
        let config = test_config(3600);
        let result = validate_token(&config, "");
        assert_eq!(result, Err(AuthError::TokenInvalid));
    }

    #[test]
    fn short_secret_is_rejected() {
        let result = JwtConfig::builder("too-short").build();
        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(matches!(err, AuthError::ConfigInvalid(_)));
    }

    #[test]
    fn debug_does_not_leak_secret() {
        let config = test_config(3600);
        let debug = format!("{config:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains(TEST_SECRET));
    }
}
