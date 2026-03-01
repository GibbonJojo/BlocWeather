#!/bin/bash

# Seed test data for BlocWeather
# First, create an admin user and get a token

API_URL="http://localhost:3000/api/v1"

echo "🔐 Creating admin user..."
psql postgresql://blocweather:blocweather@localhost/blocweather <<EOF
INSERT INTO admin_users (username, password_hash, email)
VALUES (
    'admin',
    '\$argon2id\$v=19\$m=19456,t=2,p=1\$VGhpc0lzQVNhbHRGb3JUZXN0\$8K0J9K4Z5R3M2L1N0P8Q6W5E4R3T2Y1U',
    'admin@blocweather.com'
)
ON CONFLICT (username) DO NOTHING;
EOF

echo "🔑 Logging in to get JWT token..."
TOKEN=$(curl -s -X POST "$API_URL/admin/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}' | jq -r '.token')

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
  echo "❌ Failed to get token. Make sure backend is running!"
  exit 1
fi

echo "✓ Got token: ${TOKEN:0:20}..."

# Create France
echo ""
echo "🇫🇷 Creating France..."
FRANCE_ID=$(curl -s -X POST "$API_URL/admin/countries" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"France","code":"FR"}' | jq -r '.id')

echo "✓ France ID: $FRANCE_ID"

# Create Île-de-France subregion
echo ""
echo "📍 Creating Île-de-France..."
IDF_ID=$(curl -s -X POST "$API_URL/admin/subregions" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Île-de-France\",\"country_id\":\"$FRANCE_ID\"}" | jq -r '.id')

echo "✓ Île-de-France ID: $IDF_ID"

# Create Fontainebleau spot
echo ""
echo "🧗 Creating Fontainebleau spot..."
FONT_ID=$(curl -s -X POST "$API_URL/admin/spots" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\":\"Fontainebleau\",
    \"latitude\":48.404,
    \"longitude\":2.703,
    \"country_id\":\"$FRANCE_ID\",
    \"subregion_id\":\"$IDF_ID\",
    \"description\":\"World-class bouldering in beautiful forest setting\",
    \"elevation_meters\":100,
    \"rock_type\":\"sandstone\",
    \"exposure\":\"varied\"
  }" | jq -r '.id')

echo "✓ Fontainebleau ID: $FONT_ID"

# Create USA
echo ""
echo "🇺🇸 Creating USA..."
USA_ID=$(curl -s -X POST "$API_URL/admin/countries" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"United States","code":"US"}' | jq -r '.id')

echo "✓ USA ID: $USA_ID"

# Create California subregion
echo ""
echo "📍 Creating California..."
CA_ID=$(curl -s -X POST "$API_URL/admin/subregions" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"California\",\"country_id\":\"$USA_ID\"}" | jq -r '.id')

echo "✓ California ID: $CA_ID"

# Create Bishop spot
echo ""
echo "🧗 Creating Bishop (Buttermilks)..."
BISHOP_ID=$(curl -s -X POST "$API_URL/admin/spots" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\":\"Buttermilks (Bishop)\",
    \"latitude\":37.360,
    \"longitude\":-118.550,
    \"country_id\":\"$USA_ID\",
    \"subregion_id\":\"$CA_ID\",
    \"description\":\"High desert bouldering paradise\",
    \"elevation_meters\":1500,
    \"rock_type\":\"granite\",
    \"exposure\":\"S\"
  }" | jq -r '.id')

echo "✓ Bishop ID: $BISHOP_ID"

echo ""
echo "✅ Test data created successfully!"
echo ""
echo "📊 Summary:"
echo "  - 2 countries (France, USA)"
echo "  - 2 subregions (Île-de-France, California)"
echo "  - 2 spots (Fontainebleau, Bishop)"
echo ""
echo "⏳ ETL will run in the next cycle (check logs in ~30 minutes or restart backend)"
echo ""
echo "🌐 Visit: http://localhost:5173"
