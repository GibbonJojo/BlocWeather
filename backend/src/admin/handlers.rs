use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use crate::state::AppState;
use uuid::Uuid;

use super::auth::{generate_token, verify_credentials, LoginRequest, LoginResponse};

// ==================== Authentication ====================

pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let db = &state.db;
    let (user_id, username) = verify_credentials(db, &req.username, &req.password).await?;

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-change-in-production".to_string());

    let token = generate_token(&user_id, &username, &jwt_secret)
        .map_err(|e| {
            tracing::error!("Token generation error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Token generation failed".to_string())
        })?;

    Ok(Json(LoginResponse { token, username }))
}

// ==================== Spots CRUD ====================

#[derive(Deserialize)]
pub struct CreateSpotRequest {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country_id: Uuid,
    pub subregion_id: Option<Uuid>,
    pub description: Option<String>,
    pub elevation_meters: Option<i32>,
    pub rock_type: Option<String>,
    pub exposure: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSpotRequest {
    pub name: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub country_id: Option<Uuid>,
    pub subregion_id: Option<Uuid>,
    pub description: Option<String>,
    pub elevation_meters: Option<i32>,
    pub rock_type: Option<String>,
    pub exposure: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct SpotResponse {
    pub id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country_id: Uuid,
    pub subregion_id: Option<Uuid>,
    pub description: Option<String>,
    pub elevation_meters: Option<i32>,
    pub rock_type: Option<String>,
    pub exposure: Option<String>,
}

pub async fn create_spot_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateSpotRequest>,
) -> Result<Json<SpotResponse>, (StatusCode, String)> {
    let db = &state.db;

    let rock_type = req.rock_type.as_deref().unwrap_or("unknown");
    let exposure = req.exposure.as_deref().unwrap_or("varied");

    let row = sqlx::query(
        r#"
        INSERT INTO spots (
            name, location, latitude, longitude, country_id, subregion_id,
            description, elevation_meters, rock_type, exposure
        )
        VALUES ($1, ST_SetSRID(ST_MakePoint($3, $2), 4326)::geography, $2, $3, $4, $5, $6, $7, $8::rock_type, $9::exposure_type)
        RETURNING id, name, latitude, longitude, country_id, subregion_id,
                  description, elevation_meters,
                  rock_type::text as rock_type,
                  exposure::text as exposure
        "#
    )
    .bind(&req.name)
    .bind(req.latitude)
    .bind(req.longitude)
    .bind(req.country_id)
    .bind(req.subregion_id)
    .bind(&req.description)
    .bind(req.elevation_meters)
    .bind(rock_type)
    .bind(exposure)
    .fetch_one(db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    Ok(Json(SpotResponse {
        id: row.get("id"),
        name: row.get("name"),
        latitude: row.get("latitude"),
        longitude: row.get("longitude"),
        country_id: row.get("country_id"),
        subregion_id: row.get("subregion_id"),
        description: row.get("description"),
        elevation_meters: row.get("elevation_meters"),
        rock_type: row.get("rock_type"),
        exposure: row.get("exposure"),
    }))
}

pub async fn update_spot_handler(
    State(state): State<AppState>,
    Path(spot_id): Path<Uuid>,
    Json(req): Json<UpdateSpotRequest>,
) -> Result<Json<SpotResponse>, (StatusCode, String)> {
    let db = &state.db;

    let existing = sqlx::query_as::<_, SpotResponse>(
        r#"
        SELECT id, name, latitude, longitude, country_id, subregion_id,
               description, elevation_meters,
               rock_type::text as rock_type,
               exposure::text as exposure
        FROM spots
        WHERE id = $1
        "#
    )
    .bind(spot_id)
    .fetch_optional(db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Spot not found".to_string()))?;

    let name = req.name.unwrap_or(existing.name);
    let latitude = req.latitude.unwrap_or(existing.latitude);
    let longitude = req.longitude.unwrap_or(existing.longitude);
    let country_id = req.country_id.unwrap_or(existing.country_id);
    let subregion_id = req.subregion_id.or(existing.subregion_id);
    let description = req.description.or(existing.description);
    let elevation_meters = req.elevation_meters.or(existing.elevation_meters);
    let rock_type = req.rock_type.or(existing.rock_type).unwrap_or_else(|| "unknown".to_string());
    let exposure = req.exposure.or(existing.exposure).unwrap_or_else(|| "varied".to_string());

    let row = sqlx::query(
        r#"
        UPDATE spots
        SET name = $1,
            location = ST_SetSRID(ST_MakePoint($3, $2), 4326)::geography,
            latitude = $2, longitude = $3, country_id = $4,
            subregion_id = $5, description = $6, elevation_meters = $7,
            rock_type = $8::rock_type, exposure = $9::exposure_type,
            updated_at = NOW()
        WHERE id = $10
        RETURNING id, name, latitude, longitude, country_id, subregion_id,
                  description, elevation_meters,
                  rock_type::text as rock_type,
                  exposure::text as exposure
        "#
    )
    .bind(&name)
    .bind(latitude)
    .bind(longitude)
    .bind(country_id)
    .bind(subregion_id)
    .bind(&description)
    .bind(elevation_meters)
    .bind(&rock_type)
    .bind(&exposure)
    .bind(spot_id)
    .fetch_one(db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(SpotResponse {
        id: row.get("id"),
        name: row.get("name"),
        latitude: row.get("latitude"),
        longitude: row.get("longitude"),
        country_id: row.get("country_id"),
        subregion_id: row.get("subregion_id"),
        description: row.get("description"),
        elevation_meters: row.get("elevation_meters"),
        rock_type: row.get("rock_type"),
        exposure: row.get("exposure"),
    }))
}

pub async fn delete_spot_handler(
    State(state): State<AppState>,
    Path(spot_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db = &state.db;
    let result = sqlx::query("DELETE FROM spots WHERE id = $1")
        .bind(spot_id)
        .execute(db)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Spot not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, sqlx::FromRow)]
pub struct AdminSpotItem {
    pub id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country_id: Uuid,
    pub country_name: String,
    pub subregion_id: Option<Uuid>,
    pub subregion_name: Option<String>,
    pub rock_type: Option<String>,
    pub exposure: Option<String>,
    pub elevation_meters: Option<i32>,
    pub description: Option<String>,
}

pub async fn list_spots_admin_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminSpotItem>>, (StatusCode, String)> {
    let spots = sqlx::query_as::<_, AdminSpotItem>(
        r#"
        SELECT
            s.id, s.name, s.latitude, s.longitude,
            c.id AS country_id, c.name AS country_name,
            sr.id AS subregion_id, sr.name AS subregion_name,
            s.rock_type::text AS rock_type,
            s.exposure::text AS exposure,
            s.elevation_meters, s.description
        FROM spots s
        JOIN countries c ON s.country_id = c.id
        LEFT JOIN subregions sr ON s.subregion_id = sr.id
        ORDER BY c.name, s.name
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(spots))
}

// ==================== Countries CRUD ====================

#[derive(Deserialize)]
pub struct CreateCountryRequest {
    pub name: String,
    pub code: String,
}

#[derive(Deserialize)]
pub struct UpdateCountryRequest {
    pub name: Option<String>,
    pub code: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct CountryResponse {
    pub id: Uuid,
    pub name: String,
    pub code: String,
}

pub async fn create_country_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateCountryRequest>,
) -> Result<Json<CountryResponse>, (StatusCode, String)> {
    let db = &state.db;
    let country = sqlx::query_as::<_, CountryResponse>(
        "INSERT INTO countries (name, code) VALUES ($1, $2) RETURNING id, name, code"
    )
    .bind(&req.name)
    .bind(&req.code)
    .fetch_one(db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(country))
}

pub async fn update_country_handler(
    State(state): State<AppState>,
    Path(country_id): Path<Uuid>,
    Json(req): Json<UpdateCountryRequest>,
) -> Result<Json<CountryResponse>, (StatusCode, String)> {
    let db = &state.db;

    let existing = sqlx::query_as::<_, CountryResponse>(
        "SELECT id, name, code FROM countries WHERE id = $1"
    )
    .bind(country_id)
    .fetch_optional(db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Country not found".to_string()))?;

    let name = req.name.unwrap_or(existing.name);
    let code = req.code.unwrap_or(existing.code);

    let row = sqlx::query_as::<_, CountryResponse>(
        "UPDATE countries SET name = $1, code = $2 WHERE id = $3 RETURNING id, name, code"
    )
    .bind(&name)
    .bind(&code)
    .bind(country_id)
    .fetch_one(db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(row))
}

pub async fn delete_country_handler(
    State(state): State<AppState>,
    Path(country_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db = &state.db;
    let result = sqlx::query("DELETE FROM countries WHERE id = $1")
        .bind(country_id)
        .execute(db)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Country not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Subregions CRUD ====================

#[derive(Deserialize)]
pub struct CreateSubregionRequest {
    pub name: String,
    pub country_id: Uuid,
}

#[derive(Deserialize)]
pub struct UpdateSubregionRequest {
    pub name: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct SubregionResponse {
    pub id: Uuid,
    pub name: String,
    pub country_id: Uuid,
}

pub async fn create_subregion_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateSubregionRequest>,
) -> Result<Json<SubregionResponse>, (StatusCode, String)> {
    let db = &state.db;
    let subregion = sqlx::query_as::<_, SubregionResponse>(
        "INSERT INTO subregions (name, country_id) VALUES ($1, $2) RETURNING id, name, country_id"
    )
    .bind(&req.name)
    .bind(req.country_id)
    .fetch_one(db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(subregion))
}

pub async fn update_subregion_handler(
    State(state): State<AppState>,
    Path(subregion_id): Path<Uuid>,
    Json(req): Json<UpdateSubregionRequest>,
) -> Result<Json<SubregionResponse>, (StatusCode, String)> {
    let db = &state.db;

    let existing = sqlx::query_as::<_, SubregionResponse>(
        "SELECT id, name, country_id FROM subregions WHERE id = $1"
    )
    .bind(subregion_id)
    .fetch_optional(db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Subregion not found".to_string()))?;

    let name = req.name.unwrap_or(existing.name);

    let row = sqlx::query_as::<_, SubregionResponse>(
        "UPDATE subregions SET name = $1 WHERE id = $2 RETURNING id, name, country_id"
    )
    .bind(&name)
    .bind(subregion_id)
    .fetch_one(db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(row))
}

pub async fn delete_subregion_handler(
    State(state): State<AppState>,
    Path(subregion_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db = &state.db;
    let result = sqlx::query("DELETE FROM subregions WHERE id = $1")
        .bind(subregion_id)
        .execute(db)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Subregion not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, sqlx::FromRow)]
pub struct AdminSubregionItem {
    pub id: Uuid,
    pub name: String,
    pub country_id: Uuid,
    pub country_name: String,
    pub spot_count: i64,
}

pub async fn list_subregions_admin_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminSubregionItem>>, (StatusCode, String)> {
    let subs = sqlx::query_as::<_, AdminSubregionItem>(
        r#"
        SELECT
            sr.id, sr.name,
            c.id AS country_id, c.name AS country_name,
            COUNT(s.id) AS spot_count
        FROM subregions sr
        JOIN countries c ON sr.country_id = c.id
        LEFT JOIN spots s ON s.subregion_id = sr.id
        GROUP BY sr.id, sr.name, c.id, c.name
        ORDER BY c.name, sr.name
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(subs))
}

// ==================== Reports ====================

#[derive(Serialize, sqlx::FromRow)]
pub struct AdminReport {
    pub id: Uuid,
    pub spot_id: Uuid,
    pub spot_name: String,
    pub observed_at: DateTime<Utc>,
    pub status: String,
    pub reported_at: DateTime<Utc>,
    pub calc_min_saturation: Option<f32>,
    pub calc_max_saturation: Option<f32>,
}

pub async fn list_reports_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminReport>>, (StatusCode, String)> {
    let reports = sqlx::query_as::<_, AdminReport>(
        r#"
        SELECT
            cr.id, cr.spot_id,
            s.name AS spot_name,
            cr.observed_at, cr.status, cr.reported_at,
            cc.min_saturation AS calc_min_saturation,
            cc.max_saturation AS calc_max_saturation
        FROM condition_reports cr
        JOIN spots s ON cr.spot_id = s.id
        LEFT JOIN climbing_conditions cc
            ON cc.spot_id = cr.spot_id
            AND cc.timestamp = date_trunc('hour', cr.observed_at)
        ORDER BY cr.reported_at DESC
        LIMIT 200
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(reports))
}

pub async fn delete_report_handler(
    State(state): State<AppState>,
    Path(report_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM condition_reports WHERE id = $1")
        .bind(report_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Report not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
