#!/bin/bash
# ── Install systemd services and nginx config ─────────────────────────────────
# Run once as root after cloning the repo.
set -e

REPO=/opt/blocweather

cp "$REPO/deploy/blocweather-backend.service"  /etc/systemd/system/
cp "$REPO/deploy/blocweather-frontend.service" /etc/systemd/system/

cp "$REPO/deploy/nginx.conf" /etc/nginx/sites-available/blocweather
ln -sf /etc/nginx/sites-available/blocweather /etc/nginx/sites-enabled/blocweather
rm -f /etc/nginx/sites-enabled/default

systemctl daemon-reload
systemctl enable blocweather-backend blocweather-frontend

nginx -t
systemctl reload nginx

echo "Services installed. Run deploy.sh to build and start."
