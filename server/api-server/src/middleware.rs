//! JWT bearer-token middleware.
//!
//! Extracts the `Authorization: Bearer <token>` header, verifies the JWT,
//! and inserts the `JwtClaims` into request extensions for downstream handlers.
//!
//! Use `require_role!` in handlers to enforce role-based access control.

use crate::{auth::JwtClaims, error::ApiError, state::AppState};
use axum::{
    extract::{connect_info::ConnectInfo, Request, State},
    http::{header, HeaderMap},
    middleware::Next,
    response::Response,
};
use control_protocol::Role;
use std::{
    net::{IpAddr, SocketAddr},
    time::Instant,
};

fn websocket_peer_ip(req: &Request) -> Result<Option<IpAddr>, ApiError> {
    if req.uri().path() != "/ws/v1" {
        return Ok(None);
    }
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(peer)| Some(peer.ip()))
        .ok_or(ApiError::Unauthorized(
            "missing peer connection information",
        ))
}

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
    let is_websocket = req.uri().path() == "/ws/v1";
    let peer_ip = websocket_peer_ip(&req)?;
    let mut auth_attempt = peer_ip.and_then(|ip| {
        state
            .websocket_auth_failures
            .begin_attempt(ip, Instant::now())
    });
    if peer_ip.is_some() && auth_attempt.is_none() {
        return Err(ApiError::TooManyRequests);
    }
    let token = if is_websocket {
        match extract_websocket_token(req.headers()) {
            Ok(token) => token,
            Err(error) => return Err(error),
        }
    } else {
        extract_bearer(req.headers())?
    };
    let token = token.to_owned();
    let claims = state.jwt.verify(&token)?;
    if let Some(attempt) = auth_attempt.as_mut() {
        attempt.mark_success();
    }
    drop(auth_attempt);
    req.extensions_mut().insert(token);
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

fn extract_bearer(headers: &HeaderMap) -> Result<&str, ApiError> {
    let value = headers
        .get(header::AUTHORIZATION)
        .ok_or(ApiError::Unauthorized("missing Authorization header"))?
        .to_str()
        .map_err(|_| ApiError::Unauthorized("malformed Authorization header"))?;
    value
        .strip_prefix("Bearer ")
        .ok_or(ApiError::Unauthorized("expected Bearer token"))
}

const WS_AUTH_PREFIX: &str = "openiem.bearer.";
const MAX_WS_PROTOCOL_HEADER_BYTES: usize = 4096;

fn extract_websocket_token(headers: &HeaderMap) -> Result<&str, ApiError> {
    let mut protocol_values = headers.get_all(header::SEC_WEBSOCKET_PROTOCOL).iter();
    let first = protocol_values.next().ok_or(ApiError::Unauthorized(
        "missing WebSocket authentication protocol",
    ))?;
    let mut header_bytes = 0usize;
    let mut protocols = Vec::new();
    for value in std::iter::once(first).chain(protocol_values) {
        let value = value
            .to_str()
            .map_err(|_| ApiError::Unauthorized("malformed WebSocket protocol"))?;
        header_bytes = header_bytes
            .checked_add(value.len())
            .ok_or(ApiError::Unauthorized("WebSocket protocol is too large"))?;
        if header_bytes > MAX_WS_PROTOCOL_HEADER_BYTES {
            return Err(ApiError::Unauthorized("WebSocket protocol is too large"));
        }
        protocols.extend(value.split(',').map(str::trim));
    }
    let auth_tokens: Vec<&str> = protocols
        .iter()
        .filter_map(|protocol| protocol.strip_prefix(WS_AUTH_PREFIX))
        .collect();
    if protocols.len() != 2 || auth_tokens.len() != 1 || !protocols.contains(&"openiem.v1") {
        return Err(ApiError::Unauthorized(
            "invalid WebSocket authentication protocols",
        ));
    }
    let token = auth_tokens[0];
    if token.is_empty()
        || token.len() > 2048
        || token
            .chars()
            .any(|character| character.is_ascii_whitespace())
    {
        return Err(ApiError::Unauthorized(
            "invalid WebSocket authentication token",
        ));
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims_with_role(role: Role) -> JwtClaims {
        JwtClaims {
            sub: "test".to_owned(),
            user_id: 1,
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

    #[test]
    fn websocket_auth_requires_connect_info() {
        let request = Request::builder()
            .uri("/ws/v1")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(
            websocket_peer_ip(&request).unwrap_err().to_string(),
            "unauthorized: missing peer connection information"
        );
    }

    #[test]
    fn non_websocket_auth_does_not_require_connect_info() {
        let request = Request::builder()
            .uri("/api/v1/health")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(websocket_peer_ip(&request).unwrap(), None);
    }

    #[test]
    fn websocket_token_comes_from_subprotocol() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            "openiem.bearer.jwt-token, openiem.v1".parse().unwrap(),
        );
        assert_eq!(extract_websocket_token(&headers).unwrap(), "jwt-token");
    }

    #[test]
    fn websocket_token_query_is_not_accepted() {
        let headers = HeaderMap::new();
        assert!(extract_websocket_token(&headers).is_err());
    }

    #[test]
    fn websocket_token_requires_nonempty_value() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            "openiem.bearer., openiem.v1".parse().unwrap(),
        );
        assert!(extract_websocket_token(&headers).is_err());
    }

    #[test]
    fn websocket_token_rejects_duplicate_credentials() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            "openiem.bearer.one, openiem.bearer.two, openiem.v1"
                .parse()
                .unwrap(),
        );
        assert!(extract_websocket_token(&headers).is_err());
    }

    #[test]
    fn websocket_token_rejects_extra_protocols() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            "openiem.bearer.jwt-token, openiem.v1, extra"
                .parse()
                .unwrap(),
        );
        assert!(extract_websocket_token(&headers).is_err());
    }

    #[test]
    fn websocket_token_accepts_split_protocol_headers() {
        let mut headers = HeaderMap::new();
        headers.append(
            header::SEC_WEBSOCKET_PROTOCOL,
            "openiem.bearer.jwt-token".parse().unwrap(),
        );
        headers.append(
            header::SEC_WEBSOCKET_PROTOCOL,
            "openiem.v1".parse().unwrap(),
        );
        assert_eq!(extract_websocket_token(&headers).unwrap(), "jwt-token");
    }

    #[test]
    fn websocket_token_rejects_oversized_protocol_header() {
        let mut headers = HeaderMap::new();
        let oversized = format!("openiem.bearer.{}", "x".repeat(4096));
        headers.insert(header::SEC_WEBSOCKET_PROTOCOL, oversized.parse().unwrap());
        assert!(extract_websocket_token(&headers).is_err());
    }
}
