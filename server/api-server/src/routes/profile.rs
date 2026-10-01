//! Persistent musician profile endpoints.

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};
use axum::{extract::State, response::IntoResponse, Extension, Json};
use control_protocol::Role;
use serde::{Deserialize, Serialize};

const INSTRUMENTS: &[&str] = &[
    "vocals",
    "guitar",
    "bass",
    "drums",
    "keys",
    "acoustic-guitar",
    "brass",
    "strings",
];
const MIN_DISPLAY_CHARS: usize = 1;
const MAX_DISPLAY_CHARS: usize = 80;

/// Input used to create or update a musician profile.
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub struct ProfileRequest {
    pub display_name: String,
    pub instrument_id: String,
}

/// Profile returned by musician and management endpoints.
#[allow(missing_docs)]
#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub profile_id: i64,
    pub user_id: i64,
    pub username: Option<String>,
    pub display_name: String,
    pub instrument_id: String,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

fn validate_profile(request: &ProfileRequest) -> Result<(), ApiError> {
    let chars = request.display_name.chars().count();
    if !(MIN_DISPLAY_CHARS..=MAX_DISPLAY_CHARS).contains(&chars)
        || request.display_name.trim().is_empty()
        || request.display_name.chars().any(char::is_control)
    {
        return Err(ApiError::BadRequest(
            "display_name must contain 1-80 Unicode characters and no control characters"
                .to_owned(),
        ));
    }
    if !INSTRUMENTS.contains(&request.instrument_id.as_str()) {
        return Err(ApiError::BadRequest(
            "instrument_id is not in controlled catalog".to_owned(),
        ));
    }
    Ok(())
}

fn response(
    row: (i64, i64, String, String, String, i64, i64),
    username: Option<String>,
) -> ProfileResponse {
    ProfileResponse {
        profile_id: row.0,
        user_id: row.1,
        display_name: row.2,
        instrument_id: row.3,
        status: row.4,
        created_at: row.5,
        updated_at: row.6,
        username,
    }
}

/// Read caller's own musician profile.
///
/// # Errors
/// Returns `ApiError` when caller lacks musician role or profile is absent.
#[allow(clippy::unused_async)]
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    if claims.role != Role::Musician {
        return Err(ApiError::Forbidden(
            "musician profile belongs to Musician role",
        ));
    }
    state.db.get_musician_profile(claims.user_id)?.map_or_else(
        || Err(ApiError::NotFound("musician profile not found".to_owned())),
        |profile| Ok(Json(response(profile, None))),
    )
}

/// Create or update caller's own musician profile.
///
/// # Errors
/// Returns `ApiError` for invalid profile data, role, or database failure.
#[allow(clippy::unused_async)]
pub async fn put_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Json(request): Json<ProfileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if claims.role != Role::Musician {
        return Err(ApiError::Forbidden(
            "musician profile belongs to Musician role",
        ));
    }
    validate_profile(&request)?;
    state.db.upsert_musician_profile(
        claims.user_id,
        &request.display_name,
        &request.instrument_id,
    )?;
    let profile = state
        .db
        .get_musician_profile(claims.user_id)?
        .ok_or_else(|| ApiError::Internal("profile disappeared after update".to_owned()))?;
    Ok(Json(response(profile, None)))
}

/// List musician profiles for Admin and Engineer management.
///
/// # Errors
/// Returns `ApiError` when caller lacks sufficient role or database access fails.
#[allow(clippy::unused_async)]
pub async fn list_musicians(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let profiles = state
        .db
        .list_musician_profiles()?
        .into_iter()
        .map(|row| ProfileResponse {
            profile_id: row.0,
            user_id: row.1,
            display_name: row.2,
            instrument_id: row.3,
            status: row.4,
            created_at: row.5,
            updated_at: row.6,
            username: Some(row.7),
        })
        .collect::<Vec<_>>();
    Ok(Json(profiles))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_unicode_character_count_not_bytes() {
        assert!(validate_profile(&ProfileRequest {
            display_name: "é".repeat(80),
            instrument_id: "guitar".to_owned()
        })
        .is_ok());
        assert!(validate_profile(&ProfileRequest {
            display_name: "é".repeat(81),
            instrument_id: "guitar".to_owned()
        })
        .is_err());
    }

    #[test]
    fn rejects_unknown_instrument_and_controls() {
        assert!(validate_profile(&ProfileRequest {
            display_name: "Ana".to_owned(),
            instrument_id: "mandolin".to_owned()
        })
        .is_err());
        assert!(validate_profile(&ProfileRequest {
            display_name: "Ana\n".to_owned(),
            instrument_id: "guitar".to_owned()
        })
        .is_err());
    }
}
