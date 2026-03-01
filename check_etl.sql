-- Check if ETL has run and populated data

-- 1. Check weather data count
SELECT
    'weather_data' as table_name,
    COUNT(*) as total_records,
    COUNT(DISTINCT spot_id) as spots_with_data,
    MIN(fetched_at) as first_fetch,
    MAX(fetched_at) as last_fetch
FROM weather_data;

-- 2. Check climbing conditions count
SELECT
    'climbing_conditions' as table_name,
    COUNT(*) as total_records,
    COUNT(DISTINCT spot_id) as spots_with_data,
    MIN(calculated_at) as first_calculation,
    MAX(calculated_at) as last_calculation
FROM climbing_conditions;

-- 3. Check recent weather data (last 10 records)
SELECT
    s.name as spot_name,
    w.timestamp,
    w.temperature_c,
    w.humidity_percent,
    w.is_forecast,
    w.fetched_at
FROM weather_data w
JOIN spots s ON w.spot_id = s.id
ORDER BY w.fetched_at DESC
LIMIT 10;

-- 4. Check recent climbing conditions with dry rock status
SELECT
    s.name as spot_name,
    c.timestamp,
    c.dry_rock_color,
    c.rock_surface_temp_min_c,
    c.rock_surface_temp_max_c,
    c.friction_quality,
    c.calculated_at
FROM climbing_conditions c
JOIN spots s ON c.spot_id = s.id
ORDER BY c.calculated_at DESC
LIMIT 10;

-- 5. Spots without weather data (should be empty if ETL worked)
SELECT
    s.id,
    s.name,
    s.latitude,
    s.longitude
FROM spots s
LEFT JOIN weather_data w ON s.id = w.spot_id
WHERE w.spot_id IS NULL;
