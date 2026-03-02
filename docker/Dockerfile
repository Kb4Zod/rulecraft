# Rulecraft Dockerfile
# Multi-stage build for Rust + HTMX application

# Build stage - pinned version for reproducibility
FROM rust:1.85-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to pre-build dependencies
# This layer is cached until Cargo.toml or Cargo.lock changes
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    echo "fn main() {}" > src/bin/import_rules.rs && \
    cargo build --release && \
    rm -rf src

# Copy full source and assets for build
COPY src ./src
COPY templates ./templates
COPY static ./static
COPY migrations ./migrations
COPY data/rules ./rules

# Rebuild with real source (dependencies are cached)
RUN touch src/main.rs src/lib.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 rulecraft

# Copy binaries and assets
COPY --from=builder /app/target/release/rulecraft /app/rulecraft
COPY --from=builder /app/target/release/import_rules /app/import_rules
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/rules /app/rules

# Create data directory for SQLite (volume mount point)
RUN mkdir -p /app/data && chown -R rulecraft:rulecraft /app

USER rulecraft

# Set environment defaults
ENV DATABASE_URL=sqlite:./data/rulecraft.db
ENV PORT=3000
ENV RUST_LOG=rulecraft=info,tower_http=info
ENV AI_RATE_LIMIT_PER_HOUR=5
ENV SEARCH_RATE_LIMIT_PER_MINUTE=30

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["./rulecraft"]
