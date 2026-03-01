# BlocWeather 🧗‍♂️⛅

A specialized weather application for climbers and boulderers, providing rock-specific conditions including surface temperature calculations and dryness indicators.

## 🌟 Features

- **Physics-based rock surface temperature** using solar radiation and thermodynamics
- **Dry rock indicator** (green/orange/red) based on precipitation, humidity, and drying rate
- **Interactive map** with color-coded climbing spots
- **Historical + forecast data** (5 days past, 5 days future)
- **Daily weather digest cards** with expandable hourly details
- **Friction quality assessment** for optimal climbing conditions

## 🛠️ Tech Stack

- **Backend**: Rust + Axum
- **Frontend**: SvelteKit
- **Database**: PostgreSQL + PostGIS + TimescaleDB
- **Caching**: Redis
- **Maps**: Leaflet + OpenStreetMap
- **Weather Data**: Open-Meteo API

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

### 1. Rust
```bash
# Windows (using rustup)
# Download and run: https://rustup.rs/

# Verify installation
rustc --version
cargo --version
```

### 2. Node.js & npm (✅ Already installed)
```bash
node --version  # Should show v18+ or v20+
npm --version
```

### 3. PostgreSQL with PostGIS and TimescaleDB

**Option A: Using Docker (Recommended for development)**
```bash
# Install Docker Desktop for Windows: https://www.docker.com/products/docker-desktop

# Run PostgreSQL with PostGIS and TimescaleDB
docker run -d \
  --name blocweather-postgres \
  -e POSTGRES_USER=blocweather \
  -e POSTGRES_PASSWORD=blocweather \
  -e POSTGRES_DB=blocweather \
  -p 5432:5432 \
  timescale/timescaledb-ha:pg16-latest

# Connect to install PostGIS
docker exec -it blocweather-postgres psql -U blocweather -d blocweather -c "CREATE EXTENSION IF NOT EXISTS postgis;"
```

**Option B: Manual Installation on Windows**
```bash
# 1. Download and install PostgreSQL: https://www.postgresql.org/download/windows/
# 2. Download and install PostGIS: https://postgis.net/install/
# 3. Download and install TimescaleDB: https://docs.timescale.com/install/latest/installation-windows/
```

### 4. Redis

**Option A: Using Docker (Recommended)**
```bash
docker run -d --name blocweather-redis -p 6379:6379 redis:latest
```

**Option B: Manual Installation**
```bash
# Windows: Use Memurai (Redis-compatible)
# Download: https://www.memurai.com/
```

## 🚀 Quick Start

### 1. Clone and Setup

```bash
cd c:\Users\Jojo\Documents\dev\blocweather

# Backend setup
cd backend
copy .env.example .env
# Edit .env with your database credentials

# Frontend setup
cd ../frontend
copy .env.example .env
npm install
```

### 2. Database Setup

```bash
# Install sqlx-cli for migrations
cargo install sqlx-cli --no-default-features --features rustls,postgres

# Run migrations
cd backend
sqlx database create
sqlx migrate run
```

### 3. Run Development Servers

**Terminal 1 - Backend:**
```bash
cd backend
cargo run
```

**Terminal 2 - Frontend:**
```bash
cd frontend
npm run dev
```

Visit:
- Frontend: http://localhost:5173
- Backend API: http://localhost:3000
- Health check: http://localhost:3000/health

## 📁 Project Structure

```
blocweather/
├── backend/                    # Rust + Axum backend
│   ├── migrations/            # Database migrations
│   ├── src/
│   │   ├── api/              # REST endpoints
│   │   ├── calculations/     # Physics algorithms
│   │   ├── jobs/             # Background ETL tasks
│   │   ├── models/           # Database models
│   │   ├── services/         # Business logic
│   │   └── main.rs           # Entry point
│   └── Cargo.toml
├── frontend/                   # SvelteKit frontend
│   ├── src/
│   │   ├── routes/           # Pages and routing
│   │   └── lib/
│   │       ├── components/   # Reusable UI components
│   │       ├── stores/       # State management
│   │       └── api/          # API client
│   └── package.json
└── README.md
```

## 🧪 Testing

```bash
# Backend tests
cd backend
cargo test

# Frontend tests
cd frontend
npm test
```

## 📖 API Documentation

Once the backend is running, API documentation will be available at:
- http://localhost:3000/docs (future feature)

### Key Endpoints:
- `GET /api/v1/countries` - List all countries
- `GET /api/v1/spots/{id}` - Get spot details with conditions
- `GET /api/v1/spots/{id}/weather` - Get weather timeline
- `GET /api/v1/spots/map` - Get map markers

## 🔧 Development Workflow

1. **Database changes**: Create new migration with `sqlx migrate add <name>`
2. **Backend changes**: Code auto-reloads with `cargo watch -x run` (install via `cargo install cargo-watch`)
3. **Frontend changes**: Vite provides hot module reloading automatically

## 📦 Deployment

See [C:\Users\Jojo\.claude\plans\nested-frolicking-blanket.md](C:\Users\Jojo\.claude\plans\nested-frolicking-blanket.md) for detailed deployment instructions.

## 🤝 Contributing

This is a personal project, but feedback and suggestions are welcome!

## 📄 License

MIT License (or your preferred license)

## 🆘 Troubleshooting

### PostgreSQL Connection Issues
```bash
# Check if PostgreSQL is running
docker ps  # If using Docker
# or
pg_isready  # If installed locally

# Check .env file has correct DATABASE_URL
```

### Redis Connection Issues
```bash
# Check if Redis is running
docker ps  # If using Docker
# or
redis-cli ping  # Should return "PONG"
```

### Rust Compilation Issues
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build
```

## 📚 Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Axum Guide](https://docs.rs/axum/latest/axum/)
- [SvelteKit Documentation](https://kit.svelte.dev/)
- [Open-Meteo API Docs](https://open-meteo.com/en/docs)
- [PostGIS Documentation](https://postgis.net/documentation/)
- [TimescaleDB Documentation](https://docs.timescale.com/)

http://localhost:5173/embed/spots/167850a8-bf32-4367-85b5-8f6f7f249467/chart

---

Built with ❤️ by climbers, for climbers
