# ARKM — Autonomous Agent Trading Cockpit

> **Built on the Sethu Bridge Architecture**  
> A production-grade, modular AI trading orchestrator with a high-performance Rust backend and a reactive TypeScript frontend.

---

## Overview

**ARKM** is an enterprise autonomous quantitative trading system composed of three specialized modules, unified by the **Sethu Bridge** — a shared orchestration layer that synchronizes state, tools, skills, and database memory across all components.

### The Three Modules

| Module | Role |
|--------|------|
| **Chat** | Multi-agent communication hub for routing prompts to local Ollama models and specialized trading agents |
| **Tredo** | High-speed trading exchange dashboard — live orderbooks, automated bots, and execution automation |
| **Tantra** | Coworker & systems cockpit — collaborative tools, alert dispatch, and systems monitoring |

---

## Architecture

```
arkm/
├── Cargo.toml                    # Root Rust workspace manifest
├── package.json                  # Root NPM monorepo (Turborepo)
├── turbo.json                    # Turborepo pipeline config
├── docker-compose.yml            # Local dev stack (Prometheus, Grafana)
├── .env.example                  # Environment variables template
├── README.md
│
├── crates/                       # Rust library crates (workspace members)
│   ├── arkm-types/               # Shared types, enums, wire contracts (Borsh schemas)
│   ├── arkm-core/                # Re-exports & shared primitives
│   ├── arkm-execution/           # Fast-path ExecutionEngine (optimistic accounting, DashMap)
│   ├── arkm-intelligence/        # Slow-path LLM pool (Semaphore-gated, CoT-safe)
│   ├── arkm-exchange/            # Binance & KuCoin WebSocket adapters + User Data Streams
│   └── arkm-tantra/              # Alert dispatcher, systems logger, metrics monitor
│
├── backend/                      # arkm-server binary package (Axum v0.7)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               # Thin entrypoint delegating to lib
│       ├── lib.rs                # Server bootstrap — spawns all engines & starts Axum
│       └── routes.rs             # HTTP REST + WebSocket upgrade handlers
│
├── frontend/                     # React + Vite + TypeScript UI
│   ├── src/
│   │   ├── app/                  # Root layout, main.tsx, index.css (Tailwind)
│   │   ├── atoms/                # Jotai atomic state (Chat, Tredo, Tantra)
│   │   ├── components/           # OrderBookCanvas, ManualOverridePanel, LivePriceTicker
│   │   ├── workers/              # Web Worker for Borsh binary stream deserialization
│   │   └── services/             # marketDataBridge, WS client services
│   └── public/
│
├── protocols/                    # Canonical wire contracts
│   ├── borsh/                    # Binary schema definitions
│   │   ├── orderbook.borsh
│   │   ├── trade.borsh
│   │   └── alert.borsh
│   └── ts/index.ts               # TypeScript type mirrors (generated from Borsh)
│
└── deployment/                   # Infrastructure
    ├── nginx.conf                # Reverse proxy + WebSocket upgrade + COOP/COEP headers
    ├── Dockerfile.backend        # Multi-stage Rust release build
    └── Dockerfile.frontend       # Static React build served via NGINX
```

---

## Technology Stack

### Backend (Rust)
| Layer | Technology |
|-------|-----------|
| HTTP Server | `axum` v0.7 |
| Async Runtime | `tokio` v1 |
| Shared State | `dashmap` (lock-free concurrent hashmap) |
| Serialization | `serde` + `borsh` v1 (binary protocol) |
| IDs & Timestamps | `uuid` v4 + `chrono` |
| Tracing | `tracing` + `tracing-subscriber` |

### Frontend (TypeScript)
| Layer | Technology |
|-------|-----------|
| Framework | React 18 + Vite |
| State | Jotai (atomic, zero re-render overhead) |
| Styling | TailwindCSS (glassmorphic dark theme) |
| OrderBook Rendering | Canvas 2D API via Web Worker (zero-copy transferable buffers) |
| Wire Protocol | Borsh binary deserialization in Worker thread |

---

## Key Design Principles

### 1. Optimistic Accounting
The `ExecutionEngine` deducts balance and registers in-flight orders **before** the exchange confirms a fill. If the order fails, a refund task restores the full amount. This gives sub-millisecond UI feedback with zero over-trading risk.

### 2. Semaphore-Gated Intelligence Pool
The `IntelligencePool` limits concurrent LLM calls via a `tokio::sync::Semaphore`. This prevents model overloading while maintaining Chain-of-Thought (CoT) safety across simultaneous agent prompts.

### 3. Zero-Copy Order Book Rendering
Binary Borsh payloads are deserialized inside a **Web Worker** and transferred to the main thread via `postMessage` with **transferable `ArrayBuffer`** ownership. The canvas renders directly from `Float64Array` buffers — zero garbage-collector pressure at 60fps.

### 4. COOP/COEP Headers (SharedArrayBuffer)
NGINX is configured with `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp` to unlock `SharedArrayBuffer` for potential shared-memory communication between main thread and workers.

---

## Running Locally

### Prerequisites
- Rust `>=1.82` (`rustup update`)
- Node.js `>=18`
- Ollama (for local LLM inference): `ollama pull qwen3.5:0.8b`

### 1. Start the Backend
```bash
cd /home/varma/Hermes/Sethu/arkm
cargo run -p arkm-server
# Listening on http://0.0.0.0:8080
```

### 2. Start the Frontend
```bash
cd /home/varma/Hermes/Sethu/arkm/frontend
npm install
npm run dev
# Serving on http://localhost:3000
```

### 3. Production (Docker)
```bash
cd /home/varma/Hermes/Sethu/arkm
docker-compose up --build
# NGINX on :80, backend on :8080 (internal)
```

---

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/health` | System health check |
| `GET` | `/api/state` | Active positions and balances |
| `POST` | `/api/chat` | Send prompt to IntelligencePool |
| `POST` | `/api/override` | Manual trade override (bypass agent) |
| `WS` | `/ws` | Live Level 2 orderbook binary stream |

---

## Environment Variables

Copy `.env.example` to `.env` and fill in your values:

```bash
cp .env.example .env
```

| Variable | Description |
|----------|-------------|
| `PORT` | HTTP server port (default: `8080`) |
| `OLLAMA_BASE_URL` | Local Ollama endpoint (default: `http://localhost:11434`) |
| `DEFAULT_MODEL` | Active LLM model name (e.g., `qwen3.5:0.8b`) |
| `DATABASE_URL` | SQLite or Postgres connection string |
| `BINANCE_API_KEY` | Binance exchange API key |
| `BINANCE_SECRET_KEY` | Binance exchange secret |
| `KUCOIN_API_KEY` | KuCoin exchange API key |
| `KUCOIN_SECRET_KEY` | KuCoin exchange secret |

---

## Workspace Crate Reference

| Crate | Path | Role |
|-------|------|------|
| `arkm-types` | `crates/arkm-types` | Shared structs, enums, command types |
| `arkm-core` | `crates/arkm-core` | Re-exports shared primitives |
| `arkm-execution` | `crates/arkm-execution` | Fast-path order execution actor |
| `arkm-intelligence` | `crates/arkm-intelligence` | Semaphore-gated LLM pool |
| `arkm-exchange` | `crates/arkm-exchange` | Binance + KuCoin WS adapters |
| `arkm-tantra` | `crates/arkm-tantra` | Alerts + systems monitoring |
| `arkm-server` | `backend/` | Axum HTTP + WebSocket server binary |

---

## Build Status

```
✅ cargo check — 0 errors, 0 warnings
✅ All 7 workspace crates compile successfully
✅ Frontend TypeScript config verified
```

---

*ARKM is part of the Hermes autonomous intelligence ecosystem.*
