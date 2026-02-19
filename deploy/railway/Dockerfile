# syntax=docker/dockerfile:1.7
#
# ZeroClaw Railway Deployment Dockerfile
# Multi-stage build optimized for Railway hosting.
#
# Stage 1 (builder): Compiles the ZeroClaw binary from source.
# Stage 2 (runtime): Minimal Debian image with only runtime dependencies.
#
# Usage:
#   Railway auto-detects this Dockerfile via railway.toml.
#   For local testing:
#     docker build -f deploy/railway/Dockerfile -t zeroclaw-railway .

# ── Stage 1: Builder ─────────────────────────────────────────
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /build

# Install build-time dependencies required by native crates.
# - pkg-config: Locates system libraries for Rust build scripts.
# - libssl-dev: OpenSSL headers (required by reqwest/rustls fallback paths).
# - build-essential: C toolchain for native crate compilation (rusqlite bundled, etc.).
# - cmake: Required by some transitive native dependencies.
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        build-essential \
        cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests first to maximize Docker layer caching.
# A dummy main.rs triggers a dependency-only build so source changes
# do not invalidate the (slow) dependency download/compile layer.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --locked 2>/dev/null || true
RUN rm -rf src

# Copy the full source tree and build the real binary.
COPY . .
# Touch main.rs so cargo detects the source change and rebuilds.
RUN touch src/main.rs
RUN cargo build --release --locked \
    && cp target/release/zeroclaw /build/zeroclaw \
    && strip /build/zeroclaw

# ── Stage 2: Runtime ─────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# Install only the minimal runtime dependencies.
# - ca-certificates: TLS root certificates for HTTPS provider/channel calls.
# - libssl3: OpenSSL runtime library.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for the runtime process.
RUN groupadd --system zeroclaw \
    && useradd --system --gid zeroclaw --create-home --home-dir /app zeroclaw

WORKDIR /app

# Create workspace and config directories with gateway config.
# allow_public_bind is required because Railway routes external traffic
# through its reverse proxy to 0.0.0.0 inside the container.
RUN mkdir -p /app/.zeroclaw /app/workspace \
    && printf 'default_temperature = 0.7\n\n[gateway]\nallow_public_bind = true\n' > /app/.zeroclaw/config.toml \
    && chmod 600 /app/.zeroclaw/config.toml \
    && chown -R zeroclaw:zeroclaw /app

# Copy the compiled binary from the builder stage.
COPY --from=builder /build/zeroclaw /usr/local/bin/zeroclaw

# Environment variables for Railway.
# ZEROCLAW_HOST: Bind to all interfaces (required for Railway port forwarding).
# ZEROCLAW_PORT: Default gateway port; Railway maps its internal PORT to this.
# HOME: Ensures config discovery works under the non-root user.
ENV ZEROCLAW_HOST=0.0.0.0
ENV ZEROCLAW_PORT=3000
ENV HOME=/app

# Expose the gateway port.
EXPOSE 3000

# Run as non-root for security.
USER zeroclaw

# Start the ZeroClaw gateway server.
# Shell form is required so $PORT is expanded at runtime.
# Railway injects PORT; falls back to ZEROCLAW_PORT (3000) if unset.
CMD zeroclaw gateway --host 0.0.0.0 --port ${PORT:-3000}
