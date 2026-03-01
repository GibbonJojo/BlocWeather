#!/bin/bash
# ── One-time database setup ───────────────────────────────────────────────────
# Run as root after setup-server.sh. Creates the DB user, database, and runs
# the migration. Safe to re-run (uses IF NOT EXISTS / OR IGNORE patterns).
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/config.sh"

echo "Creating PostgreSQL user and database..."
sudo -u postgres psql << EOF
DO \$\$ BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'blocweather') THEN
    CREATE USER blocweather WITH PASSWORD '$DB_PASSWORD';
  END IF;
END \$\$;

SELECT 'CREATE DATABASE blocweather OWNER blocweather'
  WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'blocweather')\gexec
EOF

echo "Running migration..."
sudo -u postgres psql -d blocweather \
    < "$SCRIPT_DIR/../backend/migrations/001_initial.sql"

# Allow the blocweather OS user to connect via password (for the backend)
PG_HBA=$(sudo -u postgres psql -t -c "SHOW hba_file;" | tr -d ' ')
if ! grep -q "^host.*blocweather.*md5\|^host.*blocweather.*scram" "$PG_HBA" 2>/dev/null; then
    echo "host    blocweather     blocweather     127.0.0.1/32            scram-sha-256" >> "$PG_HBA"
    sudo -u postgres psql -c "SELECT pg_reload_conf();"
fi

echo "Database is ready."
