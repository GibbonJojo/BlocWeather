-- Slugify helper: lowercase, non-alphanumeric → hyphens, trim leading/trailing hyphens
CREATE OR REPLACE FUNCTION slugify(v TEXT) RETURNS TEXT LANGUAGE SQL IMMUTABLE AS $$
  SELECT regexp_replace(
    lower(regexp_replace(v, '[^a-zA-Z0-9]+', '-', 'g')),
    '^-+|-+$', '', 'g'
  )
$$;

-- Add slug columns
ALTER TABLE countries  ADD COLUMN IF NOT EXISTS slug TEXT;
ALTER TABLE subregions ADD COLUMN IF NOT EXISTS slug TEXT;
ALTER TABLE spots      ADD COLUMN IF NOT EXISTS slug TEXT;

-- Populate countries (globally unique)
WITH ranked AS (
  SELECT id, slugify(name) AS base,
         ROW_NUMBER() OVER (PARTITION BY slugify(name) ORDER BY created_at) AS rn
  FROM countries
)
UPDATE countries c
  SET slug = CASE WHEN r.rn = 1 THEN r.base ELSE r.base || '-' || r.rn END
  FROM ranked r WHERE c.id = r.id;

-- Populate subregions (globally unique)
WITH ranked AS (
  SELECT id, slugify(name) AS base,
         ROW_NUMBER() OVER (PARTITION BY slugify(name) ORDER BY created_at) AS rn
  FROM subregions
)
UPDATE subregions s
  SET slug = CASE WHEN r.rn = 1 THEN r.base ELSE r.base || '-' || r.rn END
  FROM ranked r WHERE s.id = r.id;

-- Populate spots (globally unique)
WITH ranked AS (
  SELECT id, slugify(name) AS base,
         ROW_NUMBER() OVER (PARTITION BY slugify(name) ORDER BY created_at) AS rn
  FROM spots
)
UPDATE spots sp
  SET slug = CASE WHEN r.rn = 1 THEN r.base ELSE r.base || '-' || r.rn END
  FROM ranked r WHERE sp.id = r.id;

-- Enforce NOT NULL and UNIQUE
ALTER TABLE countries  ALTER COLUMN slug SET NOT NULL;
ALTER TABLE subregions ALTER COLUMN slug SET NOT NULL;
ALTER TABLE spots      ALTER COLUMN slug SET NOT NULL;

ALTER TABLE countries  ADD CONSTRAINT countries_slug_unique  UNIQUE (slug);
ALTER TABLE subregions ADD CONSTRAINT subregions_slug_unique UNIQUE (slug);
ALTER TABLE spots      ADD CONSTRAINT spots_slug_unique      UNIQUE (slug);
