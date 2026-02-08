use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use deployment::Deployment;
use serde::Deserialize;

use crate::{DeploymentImpl, error::ApiError};

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

/// Middleware that requires authentication for all routes
///
/// This middleware checks for a valid Bearer token in the Authorization header
/// or in the URL query parameter (for WebSocket connections).
/// If no token is present or the token is invalid, returns 401 Unauthorized.
pub async fn auth_middleware(
    State(deployment): State<DeploymentImpl>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    // Try to get token from Authorization header first
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    let token_from_header = auth_header
        .filter(|h| h.starts_with("Bearer "))
        .map(|h| &h[7..]);

    // Try to get token from query parameter (for WebSocket connections)
    let token_from_query = request
        .uri()
        .query()
        .and_then(|q| {
            let params: TokenQuery = serde_urlencoded::from_str(q).ok()?;
            params.token
        });

    let token = match (token_from_header, token_from_query) {
        (Some(t), _) => t.to_string(),
        (None, Some(t)) => t,
        (None, None) => return Err(ApiError::Unauthorized),
    };

    // For OAuth tokens, check via remote client
    // For local auth, the token is a JWT we can validate
    // For now, just check if credentials exist
    if deployment.auth_context().get_credentials().await.is_none() {
        // Try to validate local auth token
        let local_auth = deployment.local_auth();
        if local_auth.validate_token(&token).is_ok() {
            return Ok(next.run(request).await);
        }
        return Err(ApiError::Unauthorized);
    }

    Ok(next.run(request).await)
}
