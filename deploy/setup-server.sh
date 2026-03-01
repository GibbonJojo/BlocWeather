#!/bin/bash
# ── BlocWeather one-time server setup ────────────────────────────────────────
# Run as root on a fresh Ubuntu 24.04 server.
# Usage:  bash setup-server.sh
set -e

echo "=== [1/5] System update ==="
apt-get update && apt-get upgrade -y
apt-get install -y git curl nginx build-essential pkg-config libssl-dev

echo "=== [2/5] PostgreSQL 16 + PostGIS ==="
apt-get install -y postgresql-16 postgresql-16-postgis-3
systemctl enable postgresql
systemctl start postgresql

echo "=== [3/5] Node.js 22 ==="
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs

echo "=== [4/5] App user ==="
useradd -m -s /bin/bash blocweather 2>/dev/null || echo "(user already exists)"

# Allow blocweather to restart its own services without a password
echo "blocweather ALL=(root) NOPASSWD: /bin/systemctl restart blocweather-backend, /bin/systemctl restart blocweather-frontend" \
    > /etc/sudoers.d/blocweather

echo "=== [5/5] Rust + app directory ==="
su - blocweather -c 'curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
mkdir -p /opt/blocweather
chown blocweather:blocweather /opt/blocweather

echo ""
echo "=== Setup complete! Next steps: ==="
echo "  1. As blocweather:  git clone <your-repo> /opt/blocweather"
echo "  2. Copy and fill:   cp /opt/blocweather/deploy/config.sh.example /opt/blocweather/deploy/config.sh"
echo "  3. Setup database:  bash /opt/blocweather/deploy/db-start.sh"
echo "  4. Install services: bash /opt/blocweather/deploy/install-services.sh"
echo "  5. First deploy:    su - blocweather -c 'bash /opt/blocweather/deploy/deploy.sh'"
