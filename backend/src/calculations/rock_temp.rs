#[derive(Debug, Clone, Copy)]
pub enum RockType {
    Granite,
    Sandstone,
    Limestone,
    Basalt,
    Gneiss,
    Quartzite,
    Volcanic,
    Conglomerate,
    Gritstone,
    Unknown,
}

impl RockType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "granite"      => RockType::Granite,
            "sandstone"    => RockType::Sandstone,
            "limestone"    => RockType::Limestone,
            "basalt"       => RockType::Basalt,
            "gneiss"       => RockType::Gneiss,
            "quartzite"    => RockType::Quartzite,
            "volcanic"     => RockType::Volcanic,
            "conglomerate" => RockType::Conglomerate,
            "gritstone"    => RockType::Gritstone,
            _              => RockType::Unknown,
        }
    }

    /// Returns albedo (solar absorption coefficient) for rock type.
    /// Lower albedo = more heat absorption. Kept for future use.
    #[allow(dead_code)]
    pub fn albedo(&self) -> f64 {
        match self {
            RockType::Granite      => 0.25, // Light colored, reflects more
            RockType::Sandstone    => 0.35, // Medium
            RockType::Limestone    => 0.40, // Light, high reflection
            RockType::Basalt       => 0.15, // Dark, absorbs most heat
            RockType::Gneiss       => 0.25, // Similar to granite
            RockType::Quartzite    => 0.30, // Light to medium
            RockType::Volcanic     => 0.20, // Generally dark
            RockType::Conglomerate => 0.30, // Mixed composition
            RockType::Gritstone    => 0.28, // Dark coarse sandstone
            RockType::Unknown      => 0.30, // Conservative default
        }
    }
}

/// Calculate rock surface temperature.
///
/// Returns `(min_temp, max_temp)`:
/// - `min_temp` = air_temp − 2 °C  (shaded / cold side)
/// - `max_temp` = air_temp + solar gain clamped at 0
///   where solar gain = ((solar_radiation_wm2 − 150) / 900) × 15 °C
///
/// At 150 W/m² there is no warming effect; at 1050 W/m² the rock is
/// 15 °C warmer than air.
pub fn calculate_rock_surface_temp(
    air_temp_c: f64,
    solar_radiation_wm2: f64,
) -> (f64, f64) {
    let min_temp   = air_temp_c - 2.0;
    let solar_gain = ((solar_radiation_wm2 - 150.0) / 900.0 * 15.0).max(0.0);
    let max_temp   = air_temp_c + solar_gain;
    (min_temp, max_temp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_solar_gain_below_threshold() {
        let (min, max) = calculate_rock_surface_temp(15.0, 100.0);
        assert_eq!(min, 13.0);
        assert_eq!(max, 15.0); // 100 < 150 → no gain
    }

    #[test]
    fn test_full_solar_gain() {
        // At 1050 W/m²: gain = (1050 - 150) / 900 * 15 = 15
        let (min, max) = calculate_rock_surface_temp(15.0, 1050.0);
        assert_eq!(min, 13.0);
        assert!((max - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_partial_solar_gain() {
        // At 600 W/m²: gain = (600 - 150) / 900 * 15 = 7.5
        let (min, max) = calculate_rock_surface_temp(20.0, 600.0);
        assert_eq!(min, 18.0);
        assert!((max - 27.5).abs() < 0.001);
    }

    #[test]
    fn test_max_always_gte_air_temp() {
        let (_, max) = calculate_rock_surface_temp(5.0, 0.0);
        assert!(max >= 5.0);
    }
}
