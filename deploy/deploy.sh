#!/bin/bash
# ── Deploy BlocWeather ────────────────────────────────────────────────────────
# Run on the server as the blocweather user whenever you want to update.
# Usage:  bash /opt/blocweather/deploy/deploy.sh
set -e

REPO=/opt/blocweather
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/config.sh"

echo "=== [1/4] Pull latest code ==="
cd "$REPO"
git pull

echo "=== [2/4] Build backend ==="
cd "$REPO/backend"
source "$HOME/.cargo/env"
export DATABASE_URL=$(grep "^DATABASE_URL=" "$REPO/.env" | cut -d= -f2-)
cargo build --release

echo "=== [3/4] Build frontend ==="
cd "$REPO/frontend"
npm ci
PUBLIC_API_URL="/api/v1" npm run build

echo "=== [4/4] Restart services ==="
sudo systemctl restart blocweather-backend
sudo systemctl restart blocweather-frontend

echo ""
echo "=== Deployed! http://$SERVER_IP ==="
