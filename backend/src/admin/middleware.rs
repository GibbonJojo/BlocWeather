use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use super::auth::{validate_token, Claims};

/// Authentication middleware for admin routes
pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract token from "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Get JWT secret from environment
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-change-in-production".to_string());

    // Validate token and extract claims
    let claims = validate_token(token, &jwt_secret)
        .map_err(|e| {
            tracing::warn!("Invalid token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    // Store claims in request extensions for use in handlers
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// Extension trait to extract claims from request
pub trait ClaimsExt {
    fn claims(&self) -> Option<&Claims>;
}

impl ClaimsExt for Request {
    fn claims(&self) -> Option<&Claims> {
        self.extensions().get::<Claims>()
    }
}
