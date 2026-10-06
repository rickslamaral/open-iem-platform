//! Request-boundary security checks for browser-originated state changes.

use crate::error::ApiError;
use axum::http::{header, Method, Uri};
use axum::{extract::Request, middleware::Next, response::Response};

const DEFAULT_ALLOWED_ORIGINS: &[&str] = &["http://localhost", "http://127.0.0.1"];

/// Rejects browser origins that are not explicitly allowlisted.
///
/// Requests without `Origin` remain valid for native clients. Browser state-changing
/// requests must carry an origin matching `OPENIEM_ALLOWED_ORIGINS` (comma-separated).
///
/// # Errors
/// Returns `ApiError::Forbidden` when a supplied origin is invalid, missing, or not allowed.
pub async fn validate_origin(req: Request, next: Next) -> Result<Response, ApiError> {
    let origin = req.headers().get(header::ORIGIN);
    let state_changing = matches!(
        *req.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    ) || req.uri().path() == "/ws/v1";

    let cookie_authenticated = req
        .headers()
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(';')
                .any(|pair| pair.trim().starts_with("refresh_token="))
        });
    if state_changing && (origin.is_some() || cookie_authenticated) {
        let origin = origin
            .ok_or(ApiError::Forbidden(
                "Origin header required for cookie-authenticated request",
            ))?
            .to_str()
            .map_err(|_| ApiError::Forbidden("invalid Origin header"))?;
        let same_host_origin = std::env::var("OPENIEM_KIOSK_MODE")
            .is_ok_and(|value| value.eq_ignore_ascii_case("true"))
            && req
                .headers()
                .get(header::HOST)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|host| {
                    let Ok(uri) = origin.parse::<Uri>() else {
                        return false;
                    };
                    uri.scheme_str() == Some("http")
                        && uri.authority().is_some_and(|authority| {
                            authority.host().parse::<std::net::IpAddr>().is_ok()
                                && authority.as_str() == host
                        })
                });
        let allowed = same_host_origin
            || std::env::var("OPENIEM_ALLOWED_ORIGINS").ok().map_or_else(
                || DEFAULT_ALLOWED_ORIGINS.contains(&origin),
                |value| value.split(',').map(str::trim).any(|item| item == origin),
            );
        if !allowed {
            return Err(ApiError::Forbidden("origin is not allowed"));
        }
    }

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn rejects_unknown_state_changing_origin() {
        let app = Router::new()
            .route("/", post(|| async { "ok" }))
            .layer(middleware::from_fn(validate_origin));
        let response = app
            .oneshot(
                Request::post("/")
                    .header(header::ORIGIN, "https://evil.example")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
