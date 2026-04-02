//! Axum extractor for JWT-based authentication.
//!
//! Provides [`AuthenticatedUser`](crate::extractor::AuthenticatedUser), an axum `FromRequestParts` implementation that
//! extracts and validates a JWT `Bearer` token from the `Authorization` header.
//!
//! # Usage
//!
//! Add [`JwtConfig`](crate::jwt::JwtConfig) to your axum application state, then use
//! `AuthenticatedUser` as a handler parameter:
//!
//! ```rust,no_run
//! use axum::{Router, routing::get, Json};
//! use alpe_auth::extractor::AuthenticatedUser;
//! use alpe_auth::jwt::JwtConfig;
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! #[derive(Clone)]
//! struct AppState {
//!     jwt_config: Arc<JwtConfig>,
//! }
//!
//! impl alpe_auth::extractor::HasJwtConfig for AppState {
//!     fn jwt_config(&self) -> &JwtConfig {
//!         &self.jwt_config
//!     }
//! }
//!
//! async fn protected(user: AuthenticatedUser) -> Json<String> {
//!     Json(format!("Hello, {}!", user.claims().email()))
//! }
//!
//! let state = AppState {
//!     jwt_config: Arc::new(
//!         JwtConfig::builder("a-secret-key-that-is-long-enough-for-hmac")
//!             .ttl(Duration::from_secs(3600))
//!             .build()
//!             .expect("config should be valid"),
//!     ),
//! };
//!
//! let app: Router<AppState> = Router::new()
//!     .route("/protected", get(protected));
//! ```

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

use crate::jwt::{AuthError, Claims, JwtConfig, validate_token};
use crate::rbac::{Permission, can};

/// Trait for application state that provides access to [`JwtConfig`].
///
/// Implement this on your axum `State` type so that [`AuthenticatedUser`]
/// can extract the JWT secret automatically.
///
/// # Examples
///
/// ```
/// use alpe_auth::extractor::HasJwtConfig;
/// use alpe_auth::jwt::JwtConfig;
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// #[derive(Clone)]
/// struct MyState {
///     jwt: Arc<JwtConfig>,
/// }
///
/// impl HasJwtConfig for MyState {
///     fn jwt_config(&self) -> &JwtConfig {
///         &self.jwt
///     }
/// }
/// ```
pub trait HasJwtConfig: Clone + Send + Sync + 'static {
    /// Returns a reference to the JWT configuration.
    fn jwt_config(&self) -> &JwtConfig;
}

/// An authenticated user extracted from the `Authorization: Bearer <token>` header.
///
/// This type implements [`FromRequestParts`] for any state that implements
/// [`HasJwtConfig`]. If the token is missing, malformed, expired, or signed
/// with the wrong key, the request is rejected with an appropriate HTTP status.
///
/// # HTTP Status Codes
///
/// - **401 Unauthorized** — token missing, malformed, expired, or invalid signature
/// - **403 Forbidden** — token valid but insufficient permissions (via [`require_permission`](AuthenticatedUser::require_permission))
///
/// # Examples
///
/// ```rust,no_run
/// use axum::routing::get;
/// use alpe_auth::extractor::AuthenticatedUser;
///
/// async fn handler(user: AuthenticatedUser) -> String {
///     format!("user: {}", user.claims().email())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    claims: Claims,
}

impl AuthenticatedUser {
    /// Returns the validated JWT claims for this user.
    #[must_use]
    pub const fn claims(&self) -> &Claims {
        &self.claims
    }

    /// Checks whether the authenticated user has the given [`Permission`].
    ///
    /// Evaluates [`can`] for **each** of the user's roles. If at least one role
    /// grants the permission, this returns `Ok(())`.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::InsufficientPermission`] (which maps to HTTP 403)
    /// if none of the user's roles grant the requested permission.
    pub fn require_permission(&self, permission: Permission) -> Result<(), AuthError> {
        if self.claims.roles().iter().any(|r| can(*r, permission)) {
            Ok(())
        } else {
            tracing::warn!(
                user_id = %self.claims.sub(),
                permission = %permission,
                "insufficient permissions"
            );
            Err(AuthError::InsufficientPermission)
        }
    }
}

/// Converts an [`AuthError`] into a structured JSON HTTP response.
///
/// Error responses follow the format `{"error": "<message>"}` for
/// consistent API consumption.
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::TokenExpired | Self::TokenInvalid | Self::TokenMissing => {
                StatusCode::UNAUTHORIZED
            }
            Self::InsufficientPermission => StatusCode::FORBIDDEN,
            Self::ConfigInvalid(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({ "error": self.to_string() });
        (status, axum::Json(body)).into_response()
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: HasJwtConfig,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::TokenMissing)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::TokenInvalid)?;

        let claims = validate_token(state.jwt_config(), token)?;

        tracing::debug!(user_id = %claims.sub(), "authenticated user");

        Ok(Self { claims })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::jwt::issue_token;
    use crate::rbac::Role;

    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::{delete, get};
    use std::sync::Arc;
    use std::time::Duration;
    use tower::ServiceExt;
    use uuid::Uuid;

    const TEST_SECRET: &str = "integration-test-secret-long-enough-for-hmac";

    #[derive(Clone)]
    struct TestState {
        jwt_config: Arc<JwtConfig>,
    }

    impl HasJwtConfig for TestState {
        fn jwt_config(&self) -> &JwtConfig {
            &self.jwt_config
        }
    }

    fn test_state() -> TestState {
        TestState {
            jwt_config: Arc::new(
                JwtConfig::builder(TEST_SECRET)
                    .ttl(Duration::from_secs(3600))
                    .leeway(0)
                    .build()
                    .expect("test config should be valid"),
            ),
        }
    }

    fn test_app() -> Router {
        let state = test_state();
        Router::new()
            .route("/protected", get(|_user: AuthenticatedUser| async { "ok" }))
            .route(
                "/admin-only",
                delete(|user: AuthenticatedUser| async move {
                    user.require_permission(Permission::DeleteResource)
                        .map(|()| "deleted")
                }),
            )
            .with_state(state)
    }

    fn issue_test_token(roles: &[Role]) -> String {
        let config = JwtConfig::builder(TEST_SECRET)
            .ttl(Duration::from_secs(3600))
            .leeway(0)
            .build()
            .expect("test config should be valid");
        issue_token(&config, Uuid::new_v4(), "test@example.com", roles)
            .expect("token issuance should succeed")
    }

    #[tokio::test]
    async fn request_without_auth_header_returns_401() {
        let app = test_app();
        let req = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .expect("request should build");

        let response = app.oneshot(req).await.expect("request should complete");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn request_with_invalid_token_returns_401() {
        let app = test_app();
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", "Bearer garbage-token")
            .body(Body::empty())
            .expect("request should build");

        let response = app.oneshot(req).await.expect("request should complete");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn request_with_expired_token_returns_401() {
        let config = JwtConfig::builder(TEST_SECRET)
            .ttl(Duration::from_secs(0))
            .leeway(0)
            .build()
            .expect("test config should be valid");
        let token =
            issue_token(&config, Uuid::new_v4(), "x@y.com", &[Role::Admin]).expect("should issue");

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(1100));

        let app = test_app();
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .expect("request should build");

        let response = app.oneshot(req).await.expect("request should complete");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn request_with_valid_token_succeeds() {
        let token = issue_test_token(&[Role::Admin]);

        let app = test_app();
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .expect("request should build");

        let response = app.oneshot(req).await.expect("request should complete");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn request_with_insufficient_role_returns_403() {
        let token = issue_test_token(&[Role::Viewer]); // Viewer cannot delete

        let app = test_app();
        let req = Request::builder()
            .method("DELETE")
            .uri("/admin-only")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .expect("request should build");

        let response = app.oneshot(req).await.expect("request should complete");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
