-- BlocWeather Schema
-- PostgreSQL with PostGIS

CREATE EXTENSION IF NOT EXISTS postgis;

-- ── Enums ─────────────────────────────────────────────────────────────────────

CREATE TYPE rock_type AS ENUM (
    'granite', 'sandstone', 'limestone', 'basalt', 'gneiss',
    'quartzite', 'volcanic', 'conglomerate', 'gritstone', 'unknown'
);

CREATE TYPE exposure_type AS ENUM (
    'N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW', 'varied', 'unknown'
);

-- ── Core tables ───────────────────────────────────────────────────────────────

CREATE TABLE countries (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       VARCHAR(255) NOT NULL UNIQUE,
    code       CHAR(2)      NOT NULL UNIQUE,  -- ISO 3166-1 alpha-2
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX countries_code_idx ON countries(code);

CREATE TABLE subregions (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    country_id UUID         NOT NULL REFERENCES countries(id) ON DELETE CASCADE,
    name       VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE(country_id, name)
);

CREATE INDEX subregions_country_id_idx ON subregions(country_id);

CREATE TABLE spots (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name             VARCHAR(255)           NOT NULL,
    location         GEOGRAPHY(POINT, 4326) NOT NULL,
    latitude         DOUBLE PRECISION       NOT NULL,
    longitude        DOUBLE PRECISION       NOT NULL,
    country_id       UUID          NOT NULL REFERENCES countries(id) ON DELETE CASCADE,
    subregion_id     UUID          REFERENCES subregions(id) ON DELETE SET NULL,
    description      TEXT,
    elevation_meters INTEGER,
    rock_type        rock_type     DEFAULT 'unknown',
    exposure         exposure_type DEFAULT 'unknown',
    created_at       TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_latitude  CHECK (latitude  BETWEEN -90  AND 90),
    CONSTRAINT valid_longitude CHECK (longitude BETWEEN -180 AND 180),
    CONSTRAINT valid_elevation CHECK (elevation_meters IS NULL OR elevation_meters BETWEEN -500 AND 9000)
);

CREATE INDEX spots_location_idx     ON spots USING GIST(location);
CREATE INDEX spots_country_id_idx   ON spots(country_id);
CREATE INDEX spots_subregion_id_idx ON spots(subregion_id);
CREATE INDEX spots_name_idx         ON spots(name);
CREATE INDEX spots_name_search_idx  ON spots USING GIN(to_tsvector('english', name));

-- ── Time-series tables ────────────────────────────────────────────────────────

CREATE TABLE weather_data (
    spot_id                UUID NOT NULL REFERENCES spots(id) ON DELETE CASCADE,
    timestamp              TIMESTAMPTZ NOT NULL,
    temperature_c          REAL    NOT NULL,
    dewpoint_c             REAL,
    humidity_percent       INTEGER NOT NULL CHECK (humidity_percent BETWEEN 0 AND 100),
    precipitation_mm       REAL    NOT NULL DEFAULT 0 CHECK (precipitation_mm >= 0),
    cloud_cover_percent    INTEGER NOT NULL CHECK (cloud_cover_percent BETWEEN 0 AND 100),
    wind_speed_kmh         REAL    NOT NULL CHECK (wind_speed_kmh >= 0),
    wind_direction_degrees INTEGER CHECK (wind_direction_degrees IS NULL OR wind_direction_degrees BETWEEN 0 AND 359),
    solar_radiation_wm2    REAL    CHECK (solar_radiation_wm2  IS NULL OR solar_radiation_wm2  >= 0),
    sunshine_duration_s    REAL    CHECK (sunshine_duration_s  IS NULL OR sunshine_duration_s  >= 0),
    pressure_hpa           REAL    CHECK (pressure_hpa         IS NULL OR pressure_hpa         >  0),
    is_forecast            BOOLEAN NOT NULL DEFAULT false,
    fetched_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (spot_id, timestamp)
);

CREATE INDEX weather_data_spot_timestamp_idx ON weather_data(spot_id, timestamp DESC);
CREATE INDEX weather_data_timestamp_idx      ON weather_data(timestamp DESC);
CREATE INDEX weather_data_forecast_idx       ON weather_data(is_forecast, timestamp DESC);

CREATE TABLE climbing_conditions (
    spot_id                 UUID NOT NULL REFERENCES spots(id) ON DELETE CASCADE,
    timestamp               TIMESTAMPTZ NOT NULL,
    rock_surface_temp_min_c REAL NOT NULL,
    rock_surface_temp_max_c REAL NOT NULL,
    min_saturation          REAL NOT NULL DEFAULT 0 CHECK (min_saturation BETWEEN 0 AND 1),
    max_saturation          REAL NOT NULL DEFAULT 0 CHECK (max_saturation BETWEEN 0 AND 1),
    calculated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (spot_id, timestamp)
);

CREATE INDEX climbing_conditions_spot_timestamp_idx ON climbing_conditions(spot_id, timestamp DESC);
CREATE INDEX climbing_conditions_timestamp_idx      ON climbing_conditions(timestamp DESC);

-- ── Condition reports (user-submitted validation data) ────────────────────────

CREATE TABLE condition_reports (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spot_id     UUID        NOT NULL REFERENCES spots(id) ON DELETE CASCADE,
    observed_at TIMESTAMPTZ NOT NULL,
    status      VARCHAR(20) NOT NULL CHECK (status IN ('dry', 'some_wet', 'mostly_wet', 'wet')),
    reported_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT observed_in_past CHECK (observed_at < reported_at)
);

CREATE INDEX condition_reports_spot_id_idx     ON condition_reports(spot_id);
CREATE INDEX condition_reports_observed_at_idx ON condition_reports(observed_at DESC);

-- ── Admin users ───────────────────────────────────────────────────────────────

CREATE TABLE admin_users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    email         VARCHAR(255) NOT NULL UNIQUE,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ
);

CREATE INDEX admin_users_username_idx ON admin_users(username);

-- ── API usage log ─────────────────────────────────────────────────────────────

CREATE TABLE api_usage_log (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint         VARCHAR(255) NOT NULL,
    spots_count      INTEGER      NOT NULL CHECK (spots_count > 0),
    response_time_ms INTEGER      NOT NULL CHECK (response_time_ms >= 0),
    success          BOOLEAN      NOT NULL,
    error_message    TEXT,
    called_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX api_usage_log_called_at_idx ON api_usage_log(called_at DESC);

-- ── Views ─────────────────────────────────────────────────────────────────────

CREATE VIEW spots_with_location AS
SELECT
    s.id, s.name, s.latitude, s.longitude,
    s.elevation_meters, s.rock_type, s.exposure, s.description,
    c.id  AS country_id,   c.name AS country_name, c.code AS country_code,
    sr.id AS subregion_id, sr.name AS subregion_name,
    s.created_at, s.updated_at
FROM spots s
JOIN countries c ON s.country_id = c.id
LEFT JOIN subregions sr ON s.subregion_id = sr.id;

CREATE VIEW current_conditions AS
SELECT DISTINCT ON (cc.spot_id)
    cc.spot_id, cc.timestamp,
    cc.rock_surface_temp_min_c, cc.rock_surface_temp_max_c,
    cc.min_saturation, cc.max_saturation,
    wd.temperature_c, wd.humidity_percent, wd.precipitation_mm,
    wd.wind_speed_kmh, wd.wind_direction_degrees
FROM climbing_conditions cc
LEFT JOIN weather_data wd
    ON cc.spot_id = wd.spot_id AND cc.timestamp = wd.timestamp
ORDER BY cc.spot_id, cc.timestamp DESC;
