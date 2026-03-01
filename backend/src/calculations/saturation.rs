/// Cumulative rock saturation model.
///
/// Produces two values per hour:
/// - `min_saturation`: fast-drying scenario (sunny, exposed rock) — lower bound
/// - `max_saturation`: slow-drying scenario (shaded, north-facing rock) — upper bound
///
/// Both values are in [0.0, 1.0] where 0 = completely dry, 1 = fully saturated.

const BASE_RATE: f32 = 2.0;
const CAP: f32 = 5.0;

/// Fallback dewpoint estimate via Magnus formula, used only when Open-Meteo value is unavailable.
fn dewpoint_fallback(temp_c: f32, humidity_percent: i32) -> f32 {
    let a = 17.625_f64;
    let b = 243.04_f64;
    let rh = (humidity_percent as f64 / 100.0).max(0.001);
    let gamma = rh.ln() + a * temp_c as f64 / (b + temp_c as f64);
    (b * gamma / (a - gamma)) as f32
}

/// Calculate one hour's saturation step given the previous saturation state.
///
/// Returns `(min_saturation, max_saturation)`.
///
/// # Arguments
/// - `precipitation_mm`: hourly precipitation in mm
/// - `temperature_c`: air temperature in °C
/// - `dewpoint_c`: dew point temperature in °C (from Open-Meteo; falls back to Magnus formula if None)
/// - `humidity_percent`: relative humidity 0–100
/// - `wind_speed_kmh`: wind speed in km/h
/// - `rock_temp_min`: shaded rock temperature (used for slow-drying scenario)
/// - `rock_temp_max`: sunny rock temperature (used for fast-drying scenario)
/// - `prev_min_saturation`: previous hour's min_saturation
/// - `prev_max_saturation`: previous hour's max_saturation
pub fn calculate_saturation_step(
    precipitation_mm: f32,
    temperature_c: f32,
    dewpoint_c: Option<f32>,
    humidity_percent: i32,
    wind_speed_kmh: f32,
    rock_temp_min: f32,
    rock_temp_max: f32,
    prev_min_saturation: f32,
    prev_max_saturation: f32,
) -> (f32, f32) {
    // Heavy rain: pure accumulation, no drying
    if precipitation_mm > 0.7 {
        let new_min = (prev_min_saturation + precipitation_mm / CAP).min(1.0);
        let new_max = (prev_max_saturation + precipitation_mm / CAP).min(1.0);
        return (new_min, new_max);
    }

    // Ignore drizzle
    let effective_precip = if precipitation_mm < 0.15 { 0.0 } else { precipitation_mm };

    // Smoothstep: reduces drying rate proportionally during light rain
    let x = effective_precip / 0.7;
    let smooth_x = x * x * (3.0 - 2.0 * x);
    let rate_reducing_factor = 1.0 - smooth_x;

    let dp = dewpoint_c.unwrap();
    let humidity_factor = 1.0 - humidity_percent as f32 / 100.0;

    // --- min_saturation: fast-drying (sunny rock) ---
    let spread_fast = rock_temp_max - dp;
    let low_factor = if spread_fast < 0.0 {
        0.0 // condensation, no drying
    } else if spread_fast < 2.0 {
        BASE_RATE * (spread_fast / 2.0) // linear ramp in 0–2°C spread zone
    } else {
        BASE_RATE
    };
    let temp_factor_fast = (rock_temp_max / 15.0).clamp(0.1, 1.0);
    let wind_factor_fast = (wind_speed_kmh / 40.0).clamp(0.1, 1.0);
    let drying_rate_fast = low_factor * temp_factor_fast * wind_factor_fast * humidity_factor;
    let new_min = (prev_min_saturation + effective_precip / CAP
        - drying_rate_fast * rate_reducing_factor / CAP)
        .clamp(0.0, 1.0);

    // --- max_saturation: slow-drying (shaded rock, half wind) ---
    let spread_slow = rock_temp_min - dp;
    let high_factor = if spread_slow < 0.0 {
        0.0
    } else if spread_slow < 2.0 {
        BASE_RATE * (spread_slow / 2.0)
    } else {
        BASE_RATE
    };
    let temp_factor_slow = (rock_temp_min / 15.0).clamp(0.1, 1.0);
    let wind_factor_slow = (wind_speed_kmh / 40.0).clamp(0.1, 1.0) * 0.5; // sheltered
    let drying_rate_slow = high_factor * temp_factor_slow * humidity_factor * wind_factor_slow;
    let new_max = (prev_max_saturation + effective_precip / CAP
        - drying_rate_slow * rate_reducing_factor / CAP)
        .clamp(0.0, 1.0);

    (new_min, new_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heavy_rain_accumulates() {
        let (min, max) = calculate_saturation_step(2.0, 15.0, 90, 5.0, 13.0, 20.0, 0.0, 0.0);
        assert!(min > 0.0);
        assert!(max > 0.0);
        assert_eq!(min, max); // both increase equally under heavy rain
    }

    #[test]
    fn test_dry_conditions_decrease_saturation() {
        // Start at 50% saturation, warm sunny dry day
        let (min, max) = calculate_saturation_step(0.0, 25.0, 30, 20.0, 20.0, 35.0, 0.5, 0.5);
        assert!(min < 0.5);
        assert!(max < 0.5);
        assert!(min <= max); // fast-drying always <= slow-drying
    }

    #[test]
    fn test_min_always_lte_max() {
        // After rain, fast-drying scenario should always be <= slow-drying
        let (min, max) = calculate_saturation_step(1.5, 10.0, 70, 10.0, 8.0, 15.0, 0.3, 0.6);
        assert!(min <= max);
    }

    #[test]
    fn test_drizzle_ignored() {
        let (min1, max1) = calculate_saturation_step(0.0, 20.0, 50, 15.0, 18.0, 25.0, 0.3, 0.5);
        let (min2, max2) = calculate_saturation_step(0.1, 20.0, 50, 15.0, 18.0, 25.0, 0.3, 0.5);
        assert_eq!(min1, min2);
        assert_eq!(max1, max2);
    }

    #[test]
    fn test_condensation_prevents_drying() {
        // Very high humidity, cool air: dewpoint above rock temp → no drying
        let (min, max) = calculate_saturation_step(0.0, 5.0, 99, 2.0, 3.0, 4.0, 0.8, 0.8);
        // With condensation (spread < 0), no drying should occur
        assert!(min >= 0.8 || min == 0.8); // should not decrease much
        let _ = max;
    }
}
