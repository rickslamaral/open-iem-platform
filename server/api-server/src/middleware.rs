//! JWT bearer-token middleware.
//!
//! Extracts the `Authorization: Bearer <token>` header, verifies the JWT,
//! and inserts the `JwtClaims` into request extensions for downstream handlers.
//!
//! Use `require_role!` in handlers to enforce role-based access control.

use crate::{auth::JwtClaims, error::ApiError, state::AppState};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use control_protocol::Role;

/// Extract and verify JWT from `Authorization: Bearer` header.
///
/// Inserts verified `JwtClaims` into request extensions.
/// Rejects with 401 if header is missing or token is invalid.
///
/// # Errors
/// Returns `ApiError::Unauthorized` on missing or invalid token.
pub async fn jwt_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_bearer(req.headers())?;
    let claims = state.jwt.verify(token)?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

/// Check that the caller has at least the given role.
///
/// Role hierarchy: Admin > Engineer > Musician.
/// Returns `ApiError::Forbidden` if the caller's role is insufficient.
///
/// # Errors
/// Returns `ApiError::Forbidden` on insufficient role.
pub fn require_min_role(claims: &JwtClaims, minimum: Role) -> Result<(), ApiError> {
    if role_level(claims.role) >= role_level(minimum) {
        Ok(())
    } else {
        Err(ApiError::Forbidden("insufficient role"))
    }
}

fn role_level(role: Role) -> u8 {
    match role {
        Role::Musician => 0,
        Role::Engineer => 1,
        Role::Admin => 2,
    }
}

fn extract_bearer(headers: &axum::http::HeaderMap) -> Result<&str, ApiError> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or(ApiError::Unauthorized("missing Authorization header"))?
        .to_str()
        .map_err(|_| ApiError::Unauthorized("malformed Authorization header"))?;
    value
        .strip_prefix("Bearer ")
        .ok_or(ApiError::Unauthorized("expected Bearer token"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims_with_role(role: Role) -> JwtClaims {
        JwtClaims {
            sub: "test".to_owned(),
            role,
            jti: "jti".to_owned(),
            iss: "iss".to_owned(),
            aud: "aud".to_owned(),
            iat: 0,
            exp: u64::MAX,
        }
    }

    #[test]
    fn admin_passes_all_levels() {
        let c = claims_with_role(Role::Admin);
        assert!(require_min_role(&c, Role::Admin).is_ok());
        assert!(require_min_role(&c, Role::Engineer).is_ok());
        assert!(require_min_role(&c, Role::Musician).is_ok());
    }

    #[test]
    fn engineer_passes_own_and_musician() {
        let c = claims_with_role(Role::Engineer);
        assert!(require_min_role(&c, Role::Engineer).is_ok());
        assert!(require_min_role(&c, Role::Musician).is_ok());
        assert!(require_min_role(&c, Role::Admin).is_err());
    }

    #[test]
    fn musician_only_passes_musician() {
        let c = claims_with_role(Role::Musician);
        assert!(require_min_role(&c, Role::Musician).is_ok());
        assert!(require_min_role(&c, Role::Engineer).is_err());
        assert!(require_min_role(&c, Role::Admin).is_err());
    }
}
