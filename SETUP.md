# BlocWeather Setup Guide 🚀

This guide will walk you through setting up your development environment for BlocWeather.

## ✅ Current Status

- [x] Node.js and npm installed
- [x] Project structure created
- [x] Configuration files created
- [ ] Rust installed
- [ ] PostgreSQL installed
- [ ] Redis installed

## 📦 Installation Steps

### Step 1: Install Rust

**Windows Installation:**

1. Download rustup installer: https://rustup.rs/
2. Run `rustup-init.exe`
3. Follow the prompts (default installation is recommended)
4. Restart your terminal
5. Verify installation:
   ```bash
   rustc --version
   cargo --version
   ```

### Step 2: Install PostgreSQL with Extensions

**Option A: Docker (Recommended for Development)**

This is the easiest way to get PostgreSQL + PostGIS + TimescaleDB working:

1. **Install Docker Desktop for Windows:**
   - Download: https://www.docker.com/products/docker-desktop
   - Install and restart your computer

2. **Start PostgreSQL with all extensions:**
   ```bash
   docker run -d \
     --name blocweather-postgres \
     -e POSTGRES_USER=blocweather \
     -e POSTGRES_PASSWORD=blocweather \
     -e POSTGRES_DB=blocweather \
     -p 5432:5432 \
     timescale/timescaledb-ha:pg16-latest
   ```

3. **Enable PostGIS extension:**
   ```bash
   docker exec -it blocweather-postgres psql -U blocweather -d blocweather -c "CREATE EXTENSION IF NOT EXISTS postgis;"
   ```

4. **Verify it's running:**
   ```bash
   docker ps  # Should show blocweather-postgres
   ```

**Option B: Manual Installation**

If you prefer not to use Docker:

1. **Install PostgreSQL:**
   - Download: https://www.postgresql.org/download/windows/
   - Choose version 15 or 16
   - Remember your password for the `postgres` user

2. **Install PostGIS:**
   - Download: https://postgis.net/install/
   - Use the Stack Builder (comes with PostgreSQL installer)
   - Select PostGIS from the "Spatial Extensions" category

3. **Install TimescaleDB:**
   - Download: https://docs.timescale.com/install/latest/installation-windows/
   - Follow the Windows installation guide
   - This can be more complex, so Docker is recommended

4. **Create database:**
   ```bash
   # Open Command Prompt or PowerShell
   psql -U postgres

   # In psql:
   CREATE DATABASE blocweather;
   CREATE USER blocweather WITH PASSWORD 'blocweather';
   GRANT ALL PRIVILEGES ON DATABASE blocweather TO blocweather;
   \c blocweather
   CREATE EXTENSION IF NOT EXISTS postgis;
   CREATE EXTENSION IF NOT EXISTS timescaledb;
   \q
   ```

### Step 3: Install Redis

**Option A: Docker (Recommended)**

```bash
docker run -d \
  --name blocweather-redis \
  -p 6379:6379 \
  redis:latest
```

Verify:
```bash
docker ps  # Should show blocweather-redis
```

**Option B: Use Memurai (Redis-compatible for Windows)**

1. Download Memurai: https://www.memurai.com/
2. Install and start the service
3. Default port: 6379

### Step 4: Set Up Backend

```bash
cd C:\Users\Jojo\Documents\dev\blocweather\backend

# Copy environment file
copy .env.example .env

# Edit .env file with your actual credentials if different from defaults
# Use your favorite text editor (VS Code, Notepad++, etc.)
code .env  # If you have VS Code

# Install sqlx-cli for database migrations
cargo install sqlx-cli --no-default-features --features rustls,postgres

# Create database (if not using Docker)
sqlx database create

# Run migrations (we'll create these next)
sqlx migrate run
```

### Step 5: Set Up Frontend

```bash
cd C:\Users\Jojo\Documents\dev\blocweather\frontend

# Copy environment file
copy .env.example .env

# Install dependencies
npm install
```

## 🧪 Verify Installation

### Test Backend:

```bash
cd backend
cargo run
```

Expected output:
```
2024-XX-XX INFO blocweather_backend: Starting BlocWeather Backend...
2024-XX-XX INFO blocweather_backend: Listening on 127.0.0.1:3000
```

Open http://localhost:3000/health in your browser - should show "OK"

### Test Frontend:

```bash
cd frontend
npm run dev
```

Expected output:
```
  VITE v5.x.x  ready in XXX ms
  ➜  Local:   http://localhost:5173/
```

## 🔧 Development Tools (Optional but Recommended)

### Cargo Watch (auto-reload on code changes)
```bash
cargo install cargo-watch

# Use instead of `cargo run`
cargo watch -x run
```

### SQLx Offline Mode (for faster compilation)
```bash
# Generate sqlx metadata
cargo sqlx prepare
```

### Useful Docker Commands

```bash
# Stop containers
docker stop blocweather-postgres blocweather-redis

# Start containers
docker start blocweather-postgres blocweather-redis

# View logs
docker logs blocweather-postgres
docker logs blocweather-redis

# Remove containers (if you want to start fresh)
docker rm -f blocweather-postgres blocweather-redis

# Connect to PostgreSQL
docker exec -it blocweather-postgres psql -U blocweather -d blocweather

# Connect to Redis
docker exec -it blocweather-redis redis-cli
```

## 🐛 Troubleshooting

### "cargo: command not found"
- Restart your terminal after installing Rust
- Add Rust to PATH: `C:\Users\<YourUsername>\.cargo\bin`

### "Cannot connect to PostgreSQL"
- Check if Docker container is running: `docker ps`
- Check if PostgreSQL service is running (if installed locally)
- Verify DATABASE_URL in `.env` file
- Test connection: `psql -U blocweather -d blocweather` (or with Docker: `docker exec -it blocweather-postgres psql -U blocweather -d blocweather`)

### "Cannot connect to Redis"
- Check if Docker container is running: `docker ps`
- Test connection: `redis-cli ping` (should return "PONG")

### "Port already in use"
- Backend (3000): `netstat -ano | findstr :3000` - find process ID and kill it
- Frontend (5173): `netstat -ano | findstr :5173` - find process ID and kill it
- PostgreSQL (5432): Check if another PostgreSQL is running
- Redis (6379): Check if another Redis/Memurai is running

### Rust compilation is slow
- This is normal for the first compile
- Subsequent compiles are much faster
- Use `cargo build --release` only for production builds

## ✅ Next Steps

Once all services are running:

1. ✅ Create database migrations ([backend/migrations/001_initial.sql](backend/migrations/001_initial.sql))
2. ✅ Implement database models
3. ✅ Build API endpoints
4. ✅ Implement physics calculations
5. ✅ Create frontend components

## 📚 Documentation Links

- **Rust**: https://doc.rust-lang.org/book/
- **Axum**: https://docs.rs/axum/latest/axum/
- **SvelteKit**: https://kit.svelte.dev/docs
- **PostgreSQL**: https://www.postgresql.org/docs/
- **Docker**: https://docs.docker.com/

## 💡 Tips

- Use **Docker** for PostgreSQL and Redis - much easier than manual installation
- Use **cargo watch** for backend development - auto-reloads on save
- Keep both terminals open (one for backend, one for frontend)
- Use VS Code with Rust Analyzer extension for great IDE support

---

Need help? Check the main [README.md](README.md) or the detailed [implementation plan](C:\Users\Jojo\.claude\plans\nested-frolicking-blanket.md).
