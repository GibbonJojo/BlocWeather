use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Open-Meteo API client for fetching weather data
pub struct WeatherFetcher {
    client: reqwest::Client,
    api_url: String,
}

#[derive(Debug, Serialize)]
pub struct WeatherRequest {
    pub latitude: String,
    pub longitude: String,
    pub hourly: String,
    pub past_days: u8,
    pub forecast_days: u8,
    pub timezone: String,
}

// Single location response from Open-Meteo
#[derive(Debug, Deserialize)]
pub struct OpenMeteoResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub hourly: Option<HourlyData>,
}

#[derive(Debug, Deserialize)]
pub struct HourlyData {
    pub time: Vec<String>,
    pub temperature_2m: Vec<Option<f32>>,
    pub dew_point_2m: Vec<Option<f32>>,
    pub relative_humidity_2m: Vec<Option<i32>>,
    pub precipitation: Vec<Option<f32>>,
    pub cloud_cover: Vec<Option<i32>>,
    pub wind_speed_10m: Vec<Option<f32>>,
    pub wind_direction_10m: Vec<Option<i32>>,
    pub shortwave_radiation: Vec<Option<f32>>,
    pub sunshine_duration: Vec<Option<f32>>,
    pub surface_pressure: Vec<Option<f32>>,
}

/// Parsed weather data for a single spot at a single time
#[derive(Debug, Clone)]
pub struct WeatherDataPoint {
    pub timestamp: DateTime<Utc>,
    pub temperature_c: f32,
    pub dewpoint_c: Option<f32>,
    pub humidity_percent: i32,
    pub precipitation_mm: f32,
    pub cloud_cover_percent: i32,
    pub wind_speed_kmh: f32,
    pub wind_direction_degrees: Option<i32>,
    pub solar_radiation_wm2: Option<f32>,
    pub sunshine_duration_s: Option<f32>,
    pub pressure_hpa: Option<f32>,
    pub is_forecast: bool,
}

/// Weather data for a single spot with all time points
#[derive(Debug)]
pub struct SpotWeatherData {
    pub latitude: f64,
    pub longitude: f64,
    pub data_points: Vec<WeatherDataPoint>,
}

impl WeatherFetcher {
    pub fn new(api_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url,
        }
    }

    /// Fetch weather data for multiple coordinates (batch request)
    /// Returns weather data grouped by coordinate
    pub async fn fetch_weather_batch(
        &self,
        coordinates: Vec<(f64, f64)>, // (latitude, longitude)
        past_days: u8,
        forecast_days: u8,
    ) -> Result<Vec<SpotWeatherData>, anyhow::Error> {
        if coordinates.is_empty() {
            return Ok(Vec::new());
        }

        let (lats, lons): (Vec<_>, Vec<_>) = coordinates.iter().cloned().unzip();

        // Convert coordinates to comma-separated strings for Open-Meteo API
        let latitude_str = lats.iter()
            .map(|lat| lat.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let longitude_str = lons.iter()
            .map(|lon| lon.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let params = WeatherRequest {
            latitude: latitude_str,
            longitude: longitude_str,
            hourly: [
                "temperature_2m",
                "dew_point_2m",
                "relative_humidity_2m",
                "precipitation",
                "cloud_cover",
                "wind_speed_10m",
                "wind_direction_10m",
                "shortwave_radiation",
                "sunshine_duration",
                "surface_pressure",
            ]
            .join(","),
            past_days,
            forecast_days,
            timezone: "UTC".to_string(),
        };

        tracing::debug!(
            "Fetching weather for {} coordinates: past_days={}, forecast_days={}",
            coordinates.len(),
            past_days,
            forecast_days
        );

        let response = self
            .client
            .get(&self.api_url)
            .query(&params)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(anyhow::anyhow!("API error {}: {}", status, text));
        }

        // When multiple coordinates are sent, Open-Meteo returns an array of responses
        let api_responses: Vec<OpenMeteoResponse> = response.json().await?;

        // Parse responses into structured data
        self.parse_batch_responses(api_responses, coordinates, past_days, forecast_days)
    }

    /// Parse batch of Open-Meteo API responses into our data structure
    fn parse_batch_responses(
        &self,
        responses: Vec<OpenMeteoResponse>,
        coordinates: Vec<(f64, f64)>,
        past_days: u8,
        _forecast_days: u8,
    ) -> Result<Vec<SpotWeatherData>, anyhow::Error> {
        if responses.len() != coordinates.len() {
            return Err(anyhow::anyhow!(
                "Response count ({}) doesn't match coordinate count ({})",
                responses.len(),
                coordinates.len()
            ));
        }

        let mut result = Vec::new();

        for (response, (_lat, _lon)) in responses.iter().zip(coordinates.iter()) {
            let hourly = response
                .hourly
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing hourly data in response"))?;

            let mut data_points = Vec::new();

            // Determine the boundary between past and forecast
            // Past data is the first (past_days * 24) hours
            let forecast_start_hour = past_days as usize * 24;

            for i in 0..hourly.time.len() {
                let time_str = &hourly.time[i];

                // Open-Meteo returns timestamps in ISO 8601 format without timezone
                // Examples: "2026-02-03T00:00" or "2026-02-03T00:00:00"
                let timestamp = if let Ok(dt) = DateTime::parse_from_rfc3339(time_str) {
                    dt.with_timezone(&Utc)
                } else if let Ok(naive_dt) = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M") {
                    DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc)
                } else if let Ok(naive_dt) = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S") {
                    DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc)
                } else {
                    return Err(anyhow::anyhow!("Failed to parse timestamp: {}", time_str));
                };

                let is_forecast = i >= forecast_start_hour;

                // Extract values with defaults for missing data
                let temperature = hourly.temperature_2m.get(i).and_then(|v| *v).unwrap_or(0.0);
                let dewpoint = hourly.dew_point_2m.get(i).and_then(|v| *v);
                let humidity = hourly
                    .relative_humidity_2m
                    .get(i)
                    .and_then(|v| *v)
                    .unwrap_or(50);
                let precipitation = hourly.precipitation.get(i).and_then(|v| *v).unwrap_or(0.0);
                let cloud_cover = hourly.cloud_cover.get(i).and_then(|v| *v).unwrap_or(50);
                let wind_speed = hourly
                    .wind_speed_10m
                    .get(i)
                    .and_then(|v| *v)
                    .unwrap_or(0.0);

                // Validate wind direction: must be 0-359 or None
                let wind_direction = hourly.wind_direction_10m.get(i).and_then(|v| *v).and_then(|deg| {
                    if deg >= 0 && deg < 360 {
                        Some(deg)
                    } else if deg == 360 {
                        Some(0) // Normalize 360° to 0°
                    } else {
                        None // Invalid value, treat as missing
                    }
                });

                let solar_radiation = hourly.shortwave_radiation.get(i).and_then(|v| *v);
                let sunshine_duration = hourly.sunshine_duration.get(i).and_then(|v| *v);
                let pressure = hourly.surface_pressure.get(i).and_then(|v| *v);

                data_points.push(WeatherDataPoint {
                    timestamp,
                    temperature_c: temperature,
                    dewpoint_c: dewpoint,
                    humidity_percent: humidity,
                    precipitation_mm: precipitation,
                    cloud_cover_percent: cloud_cover,
                    wind_speed_kmh: wind_speed,
                    wind_direction_degrees: wind_direction,
                    solar_radiation_wm2: solar_radiation,
                    sunshine_duration_s: sunshine_duration,
                    pressure_hpa: pressure,
                    is_forecast,
                });
            }

            result.push(SpotWeatherData {
                latitude: response.latitude,
                longitude: response.longitude,
                data_points,
            });
        }

        tracing::debug!("Parsed weather data for {} spots", result.len());

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_request_serialization() {
        let req = WeatherRequest {
            latitude: "48.4,49.7".to_string(),
            longitude: "2.7,11.25".to_string(),
            hourly: "temperature_2m,precipitation".to_string(),
            past_days: 5,
            forecast_days: 5,
            timezone: "UTC".to_string(),
        };

        let serialized = serde_json::to_string(&req).unwrap();
        assert!(serialized.contains("48.4"));
        assert!(serialized.contains("past_days"));
    }
}
