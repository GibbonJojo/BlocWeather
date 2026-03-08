use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use chrono::{DateTime, Utc};

mod calculations;
mod services;
mod jobs;
mod admin;
mod state;

use state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    database: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,blocweather_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    tracing::info!("🚀 Starting BlocWeather Backend...");

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Create database connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("✓ Database connected");

    // Test database connection
    sqlx::query("SELECT 1")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to query database");

    tracing::info!("✓ Database health check passed");

    // Get configuration from environment
    let open_meteo_url = std::env::var("OPEN_METEO_API_URL")
        .unwrap_or_else(|_| "https://api.open-meteo.com/v1/forecast".to_string());

    let sync_interval: u64 = std::env::var("WEATHER_SYNC_INTERVAL_MINUTES")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .unwrap_or(30);

    // Start weather sync job in background
    let scheduler = jobs::start_weather_sync_job(
        db_pool.clone(),
        open_meteo_url.clone(),
        sync_interval,
    )
    .await
    .expect("Failed to start weather sync job");

    tracing::info!("✓ Weather sync job is running");

    // Run ETL immediately on startup
    let startup_db = db_pool.clone();
    let startup_api = open_meteo_url.clone();
    tokio::spawn(async move {
        tracing::info!("🔄 Running initial weather sync on startup...");
        if let Err(e) = jobs::run_weather_sync_once(&startup_db, &startup_api).await {
            tracing::error!("Initial weather sync failed: {}", e);
        }
    });

    // Create application state (keep scheduler alive in Arc)
    let state = AppState {
        db: db_pool,
        scheduler: std::sync::Arc::new(scheduler),
    };

    // Build admin routes (protected with JWT auth)
    let admin_routes = Router::new()
        .route("/spots", get(admin::list_spots_admin_handler).post(admin::create_spot_handler))
        .route("/spots/:spot_id", put(admin::update_spot_handler).delete(admin::delete_spot_handler))
        .route("/countries", post(admin::create_country_handler))
        .route("/countries/:country_id", put(admin::update_country_handler).delete(admin::delete_country_handler))
        .route("/subregions", get(admin::list_subregions_admin_handler).post(admin::create_subregion_handler))
        .route("/subregions/:subregion_id", put(admin::update_subregion_handler).delete(admin::delete_subregion_handler))
        .route("/reports", get(admin::list_reports_handler))
        .route("/reports/:report_id", delete(admin::delete_report_handler))
        .route("/suggestions", get(admin::list_suggestions_handler).delete(admin::delete_all_suggestions_handler))
        .route("/suggestions/:suggestion_id", delete(admin::delete_suggestion_handler))
        .layer(middleware::from_fn(admin::auth_middleware));

    // Build public routes
    let public_routes = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/api/v1/countries", get(list_countries_handler))
        .route("/api/v1/countries/:country_id/subregions", get(list_subregions_handler))
        .route("/api/v1/subregions/:subregion_id/spots", get(list_spots_handler))
        .route("/api/v1/spots/:spot_id", get(get_spot_handler))
        .route("/api/v1/spots/:spot_id/weather", get(get_spot_weather_handler))
        .route("/api/v1/spots/:spot_id/conditions", get(get_spot_conditions_handler))
        .route("/api/v1/spots/map", get(get_map_spots_handler))
        .route("/api/v1/spots/:spot_id/reports", post(submit_report_handler))
        .route("/api/v1/search", get(search_handler))
        .route("/api/v1/suggestions", post(admin::submit_suggestion_handler))
        .route("/api/v1/admin/login", post(admin::login_handler))
        .route("/api/v1/data/:country", get(data_country_handler))
        .route("/api/v1/data/:country/:region", get(data_region_handler))
        .route("/api/v1/data/:country/:region/:spot", get(data_spot_handler))
        .route("/api/v1/data/:country/:region/:spot/reports", post(submit_report_by_slug_handler));

    // Combine routes
    let app = public_routes
        .nest("/api/v1/admin", admin_routes)
        .with_state(state)
        .layer(CorsLayer::permissive());

    // Start server
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);
    let host: std::net::IpAddr = std::env::var("HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
        .parse()
        .unwrap_or_else(|_| [127, 0, 0, 1].into());
    let addr = SocketAddr::from((host, port));
    tracing::info!("✓ Server listening on http://{}", addr);
    tracing::info!("✓ Health check: http://{}/health", addr);
    tracing::info!("✓ API: http://{}/api/v1/countries", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> &'static str {
    "BlocWeather API v1.0 🧗‍♂️⛅"
}

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    // Test database connection
    let db_status = match sqlx::query("SELECT 1").fetch_one(&state.db).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        database: db_status.to_string(),
    })
}

#[derive(Serialize, sqlx::FromRow)]
struct Country {
    id: sqlx::types::Uuid,
    name: String,
    code: String,
    slug: String,
    spot_count: i64,
}

async fn list_countries_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<Country>>, (StatusCode, String)> {
    let countries = sqlx::query_as::<_, Country>(
        r#"
        SELECT c.id, c.name, c.code, c.slug, COUNT(s.id) as spot_count
        FROM countries c
        LEFT JOIN spots s ON s.country_id = c.id
        GROUP BY c.id, c.name, c.code, c.slug
        ORDER BY c.name
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(countries))
}

#[derive(Serialize, sqlx::FromRow)]
struct Subregion {
    id: sqlx::types::Uuid,
    country_id: sqlx::types::Uuid,
    name: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct SubregionWithCount {
    id: sqlx::types::Uuid,
    name: String,
    slug: String,
    spot_count: i64,
}

async fn list_subregions_handler(
    State(state): State<AppState>,
    Path(country_id): Path<sqlx::types::Uuid>,
) -> Result<Json<Vec<SubregionWithCount>>, (StatusCode, String)> {
    let subregions = sqlx::query_as::<_, SubregionWithCount>(
        r#"
        SELECT
            sr.id,
            sr.name,
            sr.slug,
            COUNT(s.id) as spot_count
        FROM subregions sr
        LEFT JOIN spots s ON sr.id = s.subregion_id
        WHERE sr.country_id = $1
        GROUP BY sr.id, sr.name, sr.slug
        ORDER BY sr.name
        "#
    )
    .bind(country_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(subregions))
}

#[derive(Serialize, sqlx::FromRow)]
struct SpotListItem {
    id: sqlx::types::Uuid,
    name: String,
    slug: String,
    latitude: f64,
    longitude: f64,
    rock_type: String,
    exposure: String,
    climbing_types: Vec<String>,
}

async fn list_spots_handler(
    State(state): State<AppState>,
    Path(subregion_id): Path<sqlx::types::Uuid>,
) -> Result<Json<Vec<SpotListItem>>, (StatusCode, String)> {
    let spots = sqlx::query_as::<_, SpotListItem>(
        r#"
        SELECT
            id, name, slug, latitude, longitude,
            rock_type::text as rock_type,
            exposure::text as exposure,
            climbing_types
        FROM spots
        WHERE subregion_id = $1
        ORDER BY name
        "#
    )
    .bind(subregion_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(spots))
}

#[derive(Serialize)]
struct SpotDetail {
    id: sqlx::types::Uuid,
    name: String,
    slug: String,
    latitude: f64,
    longitude: f64,
    elevation_meters: Option<i32>,
    rock_type: String,
    exposure: String,
    description: Option<String>,
    climbing_types: Vec<String>,
    country: CountryInfo,
    subregion: Option<SubregionInfo>,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct CountryInfo {
    id: sqlx::types::Uuid,
    name: String,
    code: String,
    slug: String,
}

#[derive(Serialize)]
struct SubregionInfo {
    id: sqlx::types::Uuid,
    name: String,
    slug: String,
}

async fn get_spot_handler(
    State(state): State<AppState>,
    Path(spot_id): Path<sqlx::types::Uuid>,
) -> Result<Json<SpotDetail>, (StatusCode, String)> {
    // Use query_as with manual struct to avoid sqlx macro type inference issues
    #[derive(sqlx::FromRow)]
    struct SpotRow {
        id: sqlx::types::Uuid,
        name: String,
        slug: String,
        latitude: f64,
        longitude: f64,
        elevation_meters: Option<i32>,
        rock_type: Option<String>,
        exposure: Option<String>,
        description: Option<String>,
        created_at: DateTime<Utc>,
        country_id: sqlx::types::Uuid,
        country_name: String,
        country_code: String,
        country_slug: String,
        subregion_id: Option<sqlx::types::Uuid>,
        subregion_name: Option<String>,
        subregion_slug: Option<String>,
        climbing_types: Vec<String>,
    }

    let spot = sqlx::query_as::<_, SpotRow>(
        r#"
        SELECT
            s.id, s.name, s.slug, s.latitude, s.longitude, s.elevation_meters,
            s.rock_type::text as rock_type, s.exposure::text as exposure,
            s.description, s.created_at, s.climbing_types,
            c.id as country_id, c.name as country_name, c.code as country_code, c.slug as country_slug,
            sr.id as subregion_id, sr.name as subregion_name, sr.slug as subregion_slug
        FROM spots s
        JOIN countries c ON s.country_id = c.id
        LEFT JOIN subregions sr ON s.subregion_id = sr.id
        WHERE s.id = $1
        "#
    )
    .bind(spot_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Spot not found".to_string()))?;

    let detail = SpotDetail {
        id: spot.id,
        name: spot.name,
        slug: spot.slug,
        latitude: spot.latitude,
        longitude: spot.longitude,
        elevation_meters: spot.elevation_meters,
        rock_type: spot.rock_type.unwrap_or_else(|| "unknown".to_string()),
        exposure: spot.exposure.unwrap_or_else(|| "unknown".to_string()),
        description: spot.description,
        climbing_types: spot.climbing_types,
        country: CountryInfo {
            id: spot.country_id,
            name: spot.country_name,
            code: spot.country_code,
            slug: spot.country_slug,
        },
        subregion: match (spot.subregion_id, spot.subregion_name, spot.subregion_slug) {
            (Some(id), Some(name), Some(slug)) => Some(SubregionInfo { id, name, slug }),
            _ => None,
        },
        created_at: spot.created_at,
    };

    Ok(Json(detail))
}

// Query parameters for weather/conditions endpoints
#[derive(Deserialize)]
struct TimeRangeQuery {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
}

#[derive(Serialize, sqlx::FromRow)]
struct WeatherDataPoint {
    timestamp: DateTime<Utc>,
    temperature_c: f32,
    dewpoint_c: Option<f32>,
    humidity_percent: i32,
    precipitation_mm: f32,
    cloud_cover_percent: i32,
    wind_speed_kmh: f32,
    wind_direction_degrees: Option<i32>,
    solar_radiation_wm2: Option<f32>,
    sunshine_duration_s: Option<f32>,
    pressure_hpa: Option<f32>,
    is_forecast: bool,
}

async fn get_spot_weather_handler(
    State(state): State<AppState>,
    Path(spot_id): Path<sqlx::types::Uuid>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<Vec<WeatherDataPoint>>, (StatusCode, String)> {
    // Default to 5 days past + 5 days forecast if no range specified
    let start = params.start.unwrap_or_else(|| Utc::now() - chrono::Duration::days(5));
    let end = params.end.unwrap_or_else(|| Utc::now() + chrono::Duration::days(5));

    let weather_data = sqlx::query_as::<_, WeatherDataPoint>(
        r#"
        SELECT
            timestamp, temperature_c, dewpoint_c, humidity_percent, precipitation_mm,
            cloud_cover_percent, wind_speed_kmh, wind_direction_degrees,
            solar_radiation_wm2, sunshine_duration_s, pressure_hpa, is_forecast
        FROM weather_data
        WHERE spot_id = $1
          AND timestamp >= $2
          AND timestamp <= $3
        ORDER BY timestamp ASC
        "#
    )
    .bind(spot_id)
    .bind(start)
    .bind(end)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(weather_data))
}

#[derive(Serialize, sqlx::FromRow)]
struct ClimbingCondition {
    timestamp: DateTime<Utc>,
    rock_surface_temp_min_c: f32,
    rock_surface_temp_max_c: f32,
    min_saturation: f32,
    max_saturation: f32,
}

async fn get_spot_conditions_handler(
    State(state): State<AppState>,
    Path(spot_id): Path<sqlx::types::Uuid>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<Vec<ClimbingCondition>>, (StatusCode, String)> {
    // Default to 5 days past + 5 days forecast for display window
    let start = params.start.unwrap_or_else(|| Utc::now() - chrono::Duration::days(5));
    let end = params.end.unwrap_or_else(|| Utc::now() + chrono::Duration::days(5));

    let conditions = sqlx::query_as::<_, ClimbingCondition>(
        r#"
        SELECT
            timestamp, rock_surface_temp_min_c, rock_surface_temp_max_c,
            min_saturation, max_saturation
        FROM climbing_conditions
        WHERE spot_id = $1
          AND timestamp >= $2
          AND timestamp <= $3
        ORDER BY timestamp ASC
        "#
    )
    .bind(spot_id)
    .bind(start)
    .bind(end)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    Ok(Json(conditions))
}

// Query parameters for map endpoint
#[derive(Deserialize)]
struct MapBoundsQuery {
    sw_lat: f64,
    sw_lon: f64,
    ne_lat: f64,
    ne_lon: f64,
}

#[derive(Serialize)]
struct MapSpot {
    id: sqlx::types::Uuid,
    name: String,
    latitude: f64,
    longitude: f64,
    saturation: Option<f32>,
    country_slug: String,
    region_slug: String,
    spot_slug: String,
}

async fn get_map_spots_handler(
    State(state): State<AppState>,
    Query(bounds): Query<MapBoundsQuery>,
) -> Result<Json<Vec<MapSpot>>, (StatusCode, String)> {
    #[derive(sqlx::FromRow)]
    struct MapSpotRow {
        id: sqlx::types::Uuid,
        name: String,
        latitude: f64,
        longitude: f64,
        saturation: Option<f32>,
        country_slug: String,
        region_slug: String,
        spot_slug: String,
    }

    let spots = sqlx::query_as::<_, MapSpotRow>(
        r#"
        SELECT DISTINCT ON (s.id)
            s.id,
            s.name,
            s.latitude,
            s.longitude,
            cc.max_saturation as saturation,
            c.slug as country_slug,
            COALESCE(sr.slug, '-') as region_slug,
            s.slug as spot_slug
        FROM spots s
        LEFT JOIN countries c ON c.id = s.country_id
        LEFT JOIN subregions sr ON sr.id = s.subregion_id
        LEFT JOIN LATERAL (
            SELECT max_saturation
            FROM climbing_conditions
            WHERE spot_id = s.id
              AND timestamp >= NOW() - INTERVAL '1 hour'
              AND timestamp <= NOW() + INTERVAL '1 hour'
            ORDER BY ABS(EXTRACT(EPOCH FROM (timestamp - NOW())))
            LIMIT 1
        ) cc ON true
        WHERE s.latitude BETWEEN $1 AND $3
          AND s.longitude BETWEEN $2 AND $4
        "#
    )
    .bind(bounds.sw_lat)
    .bind(bounds.sw_lon)
    .bind(bounds.ne_lat)
    .bind(bounds.ne_lon)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    let result: Vec<MapSpot> = spots
        .into_iter()
        .map(|row| MapSpot {
            id: row.id,
            name: row.name,
            latitude: row.latitude,
            longitude: row.longitude,
            saturation: row.saturation,
            country_slug: row.country_slug,
            region_slug: row.region_slug,
            spot_slug: row.spot_slug,
        })
        .collect();

    Ok(Json(result))
}


// ── Slug utilities ─────────────────────────────────────────────────────────

fn slugify_name(name: &str) -> String {
    let lower = name.to_lowercase();
    let mut result = String::new();
    let mut prev_hyphen = false;
    for c in lower.chars() {
        if c.is_alphanumeric() {
            result.push(c);
            prev_hyphen = false;
        } else if !prev_hyphen && !result.is_empty() {
            result.push('-');
            prev_hyphen = true;
        }
    }
    result.trim_end_matches('-').to_string()
}

async fn unique_spot_slug(db: &sqlx::PgPool, base: &str) -> Result<String, (StatusCode, String)> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM spots WHERE slug = $1")
        .bind(base).fetch_one(db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;
    if count == 0 { return Ok(base.to_string()); }
    for i in 2..=99 {
        let candidate = format!("{}-{}", base, i);
        let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM spots WHERE slug = $1")
            .bind(&candidate).fetch_one(db).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;
        if c == 0 { return Ok(candidate); }
    }
    Err((StatusCode::INTERNAL_SERVER_ERROR, "Could not generate unique slug".to_string()))
}

// ── /api/v1/data handlers ──────────────────────────────────────────────────

#[derive(Serialize)]
struct SubregionSummary {
    id: sqlx::types::Uuid,
    name: String,
    slug: String,
    spot_count: i64,
}

#[derive(Serialize)]
struct CountryData {
    id: sqlx::types::Uuid,
    name: String,
    code: String,
    slug: String,
    subregions: Vec<SubregionSummary>,
}

async fn data_country_handler(
    State(state): State<AppState>,
    Path(country_slug): Path<String>,
) -> Result<Json<CountryData>, (StatusCode, String)> {
    #[derive(sqlx::FromRow)]
    struct CountryRow { id: sqlx::types::Uuid, name: String, code: String, slug: String }
    let country = sqlx::query_as::<_, CountryRow>(
        "SELECT id, name, code, slug FROM countries WHERE slug = $1"
    )
    .bind(&country_slug).fetch_optional(&state.db).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Country not found".to_string()))?;

    #[derive(sqlx::FromRow)]
    struct SrRow { id: sqlx::types::Uuid, name: String, slug: String, spot_count: i64 }
    let subregions = sqlx::query_as::<_, SrRow>(
        r#"SELECT sr.id, sr.name, sr.slug, COUNT(s.id) as spot_count
           FROM subregions sr
           LEFT JOIN spots s ON s.subregion_id = sr.id
           WHERE sr.country_id = $1
           GROUP BY sr.id, sr.name, sr.slug
           ORDER BY sr.name"#
    )
    .bind(country.id).fetch_all(&state.db).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;

    Ok(Json(CountryData {
        id: country.id, name: country.name, code: country.code, slug: country.slug,
        subregions: subregions.into_iter().map(|r| SubregionSummary {
            id: r.id, name: r.name, slug: r.slug, spot_count: r.spot_count,
        }).collect(),
    }))
}

#[derive(Serialize)]
struct SubregionData {
    id: sqlx::types::Uuid,
    name: String,
    slug: String,
    country: CountryInfo,
    spots: Vec<SpotListItem>,
}

async fn data_region_handler(
    State(state): State<AppState>,
    Path((country_slug, region_slug)): Path<(String, String)>,
) -> Result<Json<SubregionData>, (StatusCode, String)> {
    #[derive(sqlx::FromRow)]
    struct SrRow {
        id: sqlx::types::Uuid, name: String, slug: String,
        country_id: sqlx::types::Uuid, country_name: String,
        country_code: String, country_slug: String,
    }
    let sr = sqlx::query_as::<_, SrRow>(
        r#"SELECT sr.id, sr.name, sr.slug,
                  c.id as country_id, c.name as country_name,
                  c.code as country_code, c.slug as country_slug
           FROM subregions sr
           JOIN countries c ON sr.country_id = c.id
           WHERE sr.slug = $1 AND c.slug = $2"#
    )
    .bind(&region_slug).bind(&country_slug)
    .fetch_optional(&state.db).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Region not found".to_string()))?;

    let spots = sqlx::query_as::<_, SpotListItem>(
        r#"SELECT id, name, slug, latitude, longitude,
                  rock_type::text as rock_type, exposure::text as exposure, climbing_types
           FROM spots WHERE subregion_id = $1 ORDER BY name"#
    )
    .bind(sr.id).fetch_all(&state.db).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;

    Ok(Json(SubregionData {
        id: sr.id, name: sr.name, slug: sr.slug,
        country: CountryInfo { id: sr.country_id, name: sr.country_name, code: sr.country_code, slug: sr.country_slug },
        spots,
    }))
}

async fn data_spot_handler(
    State(state): State<AppState>,
    Path((country_slug, region_slug, spot_slug)): Path<(String, String, String)>,
) -> Result<Json<SpotDetail>, (StatusCode, String)> {
    #[derive(sqlx::FromRow)]
    struct SpotRow {
        id: sqlx::types::Uuid, name: String, slug: String,
        latitude: f64, longitude: f64, elevation_meters: Option<i32>,
        rock_type: Option<String>, exposure: Option<String>,
        description: Option<String>, created_at: DateTime<Utc>,
        country_id: sqlx::types::Uuid, country_name: String,
        country_code: String, country_slug: String,
        subregion_id: Option<sqlx::types::Uuid>, subregion_name: Option<String>,
        subregion_slug: Option<String>, climbing_types: Vec<String>,
    }

    let no_region = region_slug == "-";

    let sql = if no_region {
        r#"SELECT s.id, s.name, s.slug, s.latitude, s.longitude, s.elevation_meters,
                  s.rock_type::text as rock_type, s.exposure::text as exposure,
                  s.description, s.created_at, s.climbing_types,
                  c.id as country_id, c.name as country_name, c.code as country_code, c.slug as country_slug,
                  sr.id as subregion_id, sr.name as subregion_name, sr.slug as subregion_slug
           FROM spots s
           JOIN countries c ON s.country_id = c.id
           LEFT JOIN subregions sr ON s.subregion_id = sr.id
           WHERE s.slug = $1 AND c.slug = $2 AND s.subregion_id IS NULL"#
    } else {
        r#"SELECT s.id, s.name, s.slug, s.latitude, s.longitude, s.elevation_meters,
                  s.rock_type::text as rock_type, s.exposure::text as exposure,
                  s.description, s.created_at, s.climbing_types,
                  c.id as country_id, c.name as country_name, c.code as country_code, c.slug as country_slug,
                  sr.id as subregion_id, sr.name as subregion_name, sr.slug as subregion_slug
           FROM spots s
           JOIN countries c ON s.country_id = c.id
           LEFT JOIN subregions sr ON s.subregion_id = sr.id
           WHERE s.slug = $1 AND c.slug = $2 AND sr.slug = $3"#
    };

    let mut q = sqlx::query_as::<_, SpotRow>(sql)
        .bind(&spot_slug).bind(&country_slug);
    if !no_region { q = q.bind(&region_slug); }

    let spot = q.fetch_optional(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Spot not found".to_string()))?;

    Ok(Json(SpotDetail {
        id: spot.id, name: spot.name, slug: spot.slug,
        latitude: spot.latitude, longitude: spot.longitude,
        elevation_meters: spot.elevation_meters,
        rock_type: spot.rock_type.unwrap_or_else(|| "unknown".to_string()),
        exposure: spot.exposure.unwrap_or_else(|| "unknown".to_string()),
        description: spot.description, climbing_types: spot.climbing_types,
        country: CountryInfo { id: spot.country_id, name: spot.country_name, code: spot.country_code, slug: spot.country_slug },
        subregion: match (spot.subregion_id, spot.subregion_name, spot.subregion_slug) {
            (Some(id), Some(name), Some(slug)) => Some(SubregionInfo { id, name, slug }),
            _ => None,
        },
        created_at: spot.created_at,
    }))
}

#[derive(Deserialize)]
struct ReportConditionRequest {
    observed_at: DateTime<Utc>,
    status: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct ConditionReport {
    id: sqlx::types::Uuid,
    spot_id: sqlx::types::Uuid,
    observed_at: DateTime<Utc>,
    status: String,
    reported_at: DateTime<Utc>,
}


async fn do_insert_report(
    db: &sqlx::PgPool,
    spot_id: sqlx::types::Uuid,
    body: &ReportConditionRequest,
) -> Result<ConditionReport, (StatusCode, String)> {
    if body.observed_at >= Utc::now() {
        return Err((StatusCode::BAD_REQUEST, "observed_at must be in the past".to_string()));
    }
    let valid_statuses = ["dry", "some_wet", "mostly_wet", "wet"];
    if !valid_statuses.contains(&body.status.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "status must be one of: dry, some_wet, mostly_wet, wet".to_string()));
    }
    sqlx::query_as::<_, ConditionReport>(
        r#"
        INSERT INTO condition_reports (spot_id, observed_at, status)
        VALUES ($1, $2, $3)
        RETURNING id, spot_id, observed_at, status, reported_at
        "#
    )
    .bind(spot_id)
    .bind(body.observed_at)
    .bind(&body.status)
    .fetch_one(db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })
}

async fn submit_report_by_slug_handler(
    State(state): State<AppState>,
    Path((country_slug, region_slug, spot_slug)): Path<(String, String, String)>,
    Json(body): Json<ReportConditionRequest>,
) -> Result<(StatusCode, Json<ConditionReport>), (StatusCode, String)> {
    let no_region = region_slug == "-";

    let spot_id: sqlx::types::Uuid = if no_region {
        sqlx::query_scalar(
            r#"SELECT s.id FROM spots s
               JOIN countries c ON c.id = s.country_id
               WHERE c.slug = $1 AND s.slug = $2 AND s.subregion_id IS NULL"#
        )
        .bind(&country_slug)
        .bind(&spot_slug)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Spot not found".to_string()))?
    } else {
        sqlx::query_scalar(
            r#"SELECT s.id FROM spots s
               JOIN countries c ON c.id = s.country_id
               JOIN subregions sr ON sr.id = s.subregion_id
               WHERE c.slug = $1 AND sr.slug = $2 AND s.slug = $3"#
        )
        .bind(&country_slug)
        .bind(&region_slug)
        .bind(&spot_slug)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Spot not found".to_string()))?
    };

    do_insert_report(&state.db, spot_id, &body).await.map(|r| (StatusCode::CREATED, Json(r)))
}

async fn submit_report_handler(
    State(state): State<AppState>,
    Path(spot_id): Path<sqlx::types::Uuid>,
    Json(body): Json<ReportConditionRequest>,
) -> Result<(StatusCode, Json<ConditionReport>), (StatusCode, String)> {
    do_insert_report(&state.db, spot_id, &body).await.map(|r| (StatusCode::CREATED, Json(r)))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct SearchResult {
    kind: String,    // "spot" | "subregion" | "country"
    id: String,      // UUID as text
    name: String,
    context: String, // e.g. "Rheinland-Pfalz, Germany" for a spot
    url: String,     // slug-based path, e.g. "/germany/nordrhein-westfalen/hohenfels"
}

async fn search_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, String)> {
    let q = params.q.trim().to_string();
    if q.len() < 2 {
        return Ok(Json(vec![]));
    }

    let pattern = format!("%{}%", q);

    let results = sqlx::query_as::<_, SearchResult>(
        r#"
        (
            SELECT
                'spot'::text       AS kind,
                s.id::text         AS id,
                s.name             AS name,
                COALESCE(sr.name || ', ', '') || c.name AS context,
                '/' || c.slug || '/' || COALESCE(sr.slug, '-') || '/' || s.slug AS url
            FROM spots s
            JOIN countries c ON s.country_id = c.id
            LEFT JOIN subregions sr ON s.subregion_id = sr.id
            WHERE s.name ILIKE $1
            ORDER BY s.name
            LIMIT 5
        )
        UNION ALL
        (
            SELECT
                'subregion'::text  AS kind,
                sr.id::text        AS id,
                sr.name            AS name,
                c.name             AS context,
                '/' || c.slug || '/' || sr.slug AS url
            FROM subregions sr
            JOIN countries c ON sr.country_id = c.id
            WHERE sr.name ILIKE $2
            ORDER BY sr.name
            LIMIT 3
        )
        UNION ALL
        (
            SELECT
                'country'::text    AS kind,
                c.id::text         AS id,
                c.name             AS name,
                ''::text           AS context,
                '/' || c.slug      AS url
            FROM countries c
            WHERE c.name ILIKE $3
            ORDER BY c.name
            LIMIT 2
        )
        "#
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Search error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Search error".to_string())
    })?;

    Ok(Json(results))
}
