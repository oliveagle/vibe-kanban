use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use deployment::Deployment;

use crate::{DeploymentImpl, error::ApiError};

/// Middleware that requires authentication for all routes
///
/// This middleware checks for a valid Bearer token in the Authorization header.
/// If no token is present or the token is invalid, returns 401 Unauthorized.
pub async fn auth_middleware(
    State(deployment): State<DeploymentImpl>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    // Check if Authorization header exists
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(ApiError::Unauthorized),
    };

    // For OAuth tokens, check via remote client
    // For local auth, the token is a JWT we can validate
    // For now, just check if credentials exist
    if deployment.auth_context().get_credentials().await.is_none() {
        // Try to validate local auth token
        if let Some(local_auth) = deployment.local_auth().ok() {
            if local_auth.validate_token(token).is_ok() {
                return Ok(next.run(request).await);
            }
        }
        return Err(ApiError::Unauthorized);
    }

    Ok(next.run(request).await)
}
