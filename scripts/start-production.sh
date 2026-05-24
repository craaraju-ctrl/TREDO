#!/bin/bash
# ── ARKM Monorepo Production Startup Orchestrator ──────────────────────────

set -e

echo "🚀 Booting ARKM Monorepo Production Deployment Stack..."

# 1. Environment file setup
if [ ! -f .env ]; then
    echo "[Config] Creating local .env from production template..."
    cp .env.example .env
else
    echo "[Config] Local .env already exists, using active values."
fi

# 2. Check for docker daemon availability
if ! docker info >/dev/null 2>&1; then
    echo "❌ Error: Docker daemon is not running! Please start docker first."
    exit 1
fi

# 3. Pull and compile image caches
echo "[Docker] Compiling Rust and Node multi-stage build layers..."
docker compose build

# 4. Bring all services online
echo "[Docker] Arming container ecosystem..."
docker compose up -d

# 5. Output active runtime report
echo ""
echo "✨ ARKM Cockpit successfully online in production release mode!"
echo "------------------------------------------------------------"
docker compose ps
echo "------------------------------------------------------------"
echo "🌐 React Cockpit Gateway (SPA + NGINX Proxy) -> http://localhost"
echo "📊 Grafana Visualizer Risk Cockpit         -> http://localhost:9001"
echo "🔧 Axum Core REST Interface               -> http://localhost:8080"
echo "------------------------------------------------------------"
echo "Tail logs using: docker compose logs -f"
