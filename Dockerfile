# syntax=docker/dockerfile:1

# =============================================================================
# Stage 1: Build the frontend
# =============================================================================
FROM node:20-alpine AS frontend

WORKDIR /app/frontend

COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

# =============================================================================
# Stage 2: Build the Rust backend (with embedded frontend)
# =============================================================================
FROM rust:1.100-bookworm AS backend

WORKDIR /app/backend

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/migrations ./migrations
COPY backend/src ./src

# Copy pre-built frontend from stage 1 (rust-embed reads from ../frontend/dist)
COPY --from=frontend /app/frontend/dist ./../frontend/dist

# Build the release binary
RUN cargo build --release

# =============================================================================
# Stage 3: Runtime image
# =============================================================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies (ca-certificates for HTTPS, etc.)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=backend /app/backend/target/release/boron-backend ./boron-backend

# Copy example env (user should override at runtime)
COPY backend/.env.example .env

# Make binary executable
RUN chmod +x ./boron-backend

# Expose the web port (configurable via WEB_PORT env var)
EXPOSE 25004

# Run the backend (serves frontend on / and API on /api/)
CMD ["./boron-backend"]