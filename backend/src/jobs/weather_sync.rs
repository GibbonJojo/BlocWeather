use crate::calculations::{calculate_rock_surface_temp, calculate_saturation_step};
use crate::services::WeatherFetcher;
use chrono::{Utc, DateTime};
use anyhow::Result;
use sqlx::PgPool;
use std::time::Instant;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

/// Start the weather synchronization job
/// Runs every N minutes (configured via interval_minutes)
pub async fn start_weather_sync_job(
    db_pool: PgPool,
    api_url: String,
    interval_minutes: u64,
) -> Result<JobScheduler, anyhow::Error> {
    let scheduler = JobScheduler::new().await?;

    // Create cron expression (e.g., "0 */30 * * * *" for every 30 minutes)
    let cron_expr = format!("0 */{} * * * *", interval_minutes);

    tracing::info!(
        "Scheduling weather sync job: every {} minutes ({})",
        interval_minutes,
        cron_expr
    );

    let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
        let db = db_pool.clone();
        let api = api_url.clone();

        Box::pin(async move {
            if let Err(e) = run_weather_sync(&db, &api).await {
                tracing::error!("Weather sync job failed: {}", e);
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    tracing::info!("✓ Weather sync job scheduled and started");

    Ok(scheduler)
}

/// Run a single weather synchronization cycle (public wrapper for manual triggering)
pub async fn run_weather_sync_once(db: &PgPool, api_url: &str) -> Result<(), anyhow::Error> {
    run_weather_sync(db, api_url).await
}

/// Run a single weather synchronization cycle (internal)
async fn run_weather_sync(db: &PgPool, api_url: &str) -> Result<(), anyhow::Error> {
    let start = Instant::now();
    tracing::info!("🔄 Starting weather sync...");

    // Fetch all active spots
    let spots = fetch_all_spots(db).await?;
    tracing::info!("Found {} spots to update", spots.len());

    if spots.is_empty() {
        tracing::info!("No spots to update, skipping sync");
        return Ok(());
    }

    // Create weather fetcher
    let fetcher = WeatherFetcher::new(api_url.to_string());

    // Batch spots into groups of 10 (Open-Meteo supports multiple coordinates)
    let batch_size = 3;
    let mut total_updated = 0;
    let mut total_errors = 0;

    for (batch_num, batch) in spots.chunks(batch_size).enumerate() {
        tracing::debug!(
            "Processing batch {}/{} ({} spots)",
            batch_num + 1,
            (spots.len() + batch_size - 1) / batch_size,
            batch.len()
        );

        let coordinates: Vec<(f64, f64)> = batch
            .iter()
            .map(|s| (s.latitude, s.longitude))
            .collect();

        // Fetch 14 days past for accurate cumulative saturation, 5 days forecast for display
        match fetcher.fetch_weather_batch(coordinates, 14, 5).await {
            Ok(weather_data) => {
                for (spot, spot_weather) in batch.iter().zip(weather_data.iter()) {
                    match process_spot_weather(db, spot, spot_weather).await {
                        Ok(_) => total_updated += 1,
                        Err(e) => {
                            tracing::error!("Failed to process spot {}: {}", spot.name, e);
                            total_errors += 1;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to fetch weather for batch {}: {}", batch_num + 1, e);
                total_errors += batch.len();
            }
        }

        // Small delay between batches to be nice to the API
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    let duration = start.elapsed();
    tracing::info!(
        "✓ Weather sync completed in {:.2}s: {} spots updated, {} errors",
        duration.as_secs_f64(),
        total_updated,
        total_errors
    );

    Ok(())
}

/// Spot data from database
#[derive(Debug, sqlx::FromRow)]
struct SpotInfo {
    id: Uuid,
    name: String,
    latitude: f64,
    longitude: f64,
    elevation_meters: Option<i32>,
    rock_type: String,
    exposure: String,
}

/// Fetch all spots from database
async fn fetch_all_spots(db: &PgPool) -> Result<Vec<SpotInfo>, sqlx::Error> {
    sqlx::query_as::<_, SpotInfo>(
        r#"
        SELECT
            id,
            name,
            latitude,
            longitude,
            elevation_meters,
            rock_type::text as rock_type,
            exposure::text as exposure
        FROM spots
        ORDER BY created_at
        "#
    )
    .fetch_all(db)
    .await
}

/// Process weather data for a single spot.
///
/// Fetches 14 days of history so the cumulative saturation calculation is
/// properly initialised before the display window. All data points are stored
/// (the API still only returns the 5-day display window by default).
async fn process_spot_weather(
    db: &PgPool,
    spot: &SpotInfo,
    weather: &crate::services::SpotWeatherData,
) -> Result<(), anyhow::Error> {
    // Saturation state carried forward across hours
    let mut prev_min_sat = 0.0_f32;
    let mut prev_max_sat = 0.0_f32;

    for data_point in &weather.data_points {
        store_weather_data(db, spot.id, data_point).await?;

        let (rock_temp_min, rock_temp_max) = calculate_rock_surface_temp(
            data_point.temperature_c as f64,
            data_point.solar_radiation_wm2.unwrap_or(0.0) as f64,
        );

        let (min_sat, max_sat) = calculate_saturation_step(
            data_point.precipitation_mm,
            data_point.temperature_c,
            data_point.dewpoint_c,
            data_point.humidity_percent,
            data_point.wind_speed_kmh,
            rock_temp_min as f32,
            rock_temp_max as f32,
            prev_min_sat,
            prev_max_sat,
        );

        store_climbing_conditions(
            db,
            spot.id,
            data_point.timestamp,
            rock_temp_min as f32,
            rock_temp_max as f32,
            min_sat,
            max_sat,
        )
        .await?;

        prev_min_sat = min_sat;
        prev_max_sat = max_sat;
    }

    Ok(())
}

/// Store raw weather data in database
async fn store_weather_data(
    db: &PgPool,
    spot_id: Uuid,
    data: &crate::services::WeatherDataPoint,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO weather_data (
            spot_id, timestamp, temperature_c, dewpoint_c, humidity_percent,
            precipitation_mm, cloud_cover_percent, wind_speed_kmh,
            wind_direction_degrees, solar_radiation_wm2, sunshine_duration_s,
            pressure_hpa, is_forecast, fetched_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW())
        ON CONFLICT (spot_id, timestamp)
        DO UPDATE SET
            temperature_c = EXCLUDED.temperature_c,
            dewpoint_c = EXCLUDED.dewpoint_c,
            humidity_percent = EXCLUDED.humidity_percent,
            precipitation_mm = EXCLUDED.precipitation_mm,
            cloud_cover_percent = EXCLUDED.cloud_cover_percent,
            wind_speed_kmh = EXCLUDED.wind_speed_kmh,
            wind_direction_degrees = EXCLUDED.wind_direction_degrees,
            solar_radiation_wm2 = EXCLUDED.solar_radiation_wm2,
            sunshine_duration_s = EXCLUDED.sunshine_duration_s,
            pressure_hpa = EXCLUDED.pressure_hpa,
            is_forecast = EXCLUDED.is_forecast,
            fetched_at = NOW()
        "#
    )
    .bind(spot_id)
    .bind(data.timestamp)
    .bind(data.temperature_c)
    .bind(data.dewpoint_c)
    .bind(data.humidity_percent)
    .bind(data.precipitation_mm)
    .bind(data.cloud_cover_percent)
    .bind(data.wind_speed_kmh)
    .bind(data.wind_direction_degrees)
    .bind(data.solar_radiation_wm2)
    .bind(data.sunshine_duration_s)
    .bind(data.pressure_hpa)
    .bind(data.is_forecast)
    .execute(db)
    .await?;

    Ok(())
}

/// Store calculated climbing conditions in database
async fn store_climbing_conditions(
    db: &PgPool,
    spot_id: Uuid,
    timestamp: DateTime<Utc>,
    rock_temp_min: f32,
    rock_temp_max: f32,
    min_saturation: f32,
    max_saturation: f32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO climbing_conditions (
            spot_id, timestamp, rock_surface_temp_min_c, rock_surface_temp_max_c,
            min_saturation, max_saturation, calculated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        ON CONFLICT (spot_id, timestamp)
        DO UPDATE SET
            rock_surface_temp_min_c = EXCLUDED.rock_surface_temp_min_c,
            rock_surface_temp_max_c = EXCLUDED.rock_surface_temp_max_c,
            min_saturation = EXCLUDED.min_saturation,
            max_saturation = EXCLUDED.max_saturation,
            calculated_at = NOW()
        "#
    )
    .bind(spot_id)
    .bind(timestamp)
    .bind(rock_temp_min)
    .bind(rock_temp_max)
    .bind(min_saturation)
    .bind(max_saturation)
    .execute(db)
    .await?;

    Ok(())
}
