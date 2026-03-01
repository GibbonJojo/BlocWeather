#!/bin/bash
# ── Start PostgreSQL + PostGIS in Docker ──────────────────────────────────────
# Run once (or after a server reboot if the container isn't set to auto-restart).
# Requires deploy/config.sh to exist.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/config.sh"

echo "Starting blocweather-db container..."
docker run -d \
    --name blocweather-db \
    --restart unless-stopped \
    --platform linux/arm64 \
    -e POSTGRES_USER=blocweather \
    -e POSTGRES_PASSWORD="$DB_PASSWORD" \
    -e POSTGRES_DB=blocweather \
    -p 127.0.0.1:5432:5432 \
    postgis/postgis:16-3.4

echo "Waiting 15s for database to initialise..."
sleep 15

echo "Running migration..."
docker exec -i blocweather-db psql -U blocweather -d blocweather \
    < "$SCRIPT_DIR/../backend/migrations/001_initial.sql"

echo "Database is ready."
