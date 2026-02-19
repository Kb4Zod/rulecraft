# Rulecraft Dockerfile
# Multi-stage build for Rust + HTMX application

# Build stage
FROM rust:latest AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml ./

# Copy full source for build
COPY src ./src
COPY templates ./templates
COPY static ./static
COPY migrations ./migrations

# Build release binary
RUN cargo build --release

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

# Copy binary and assets
COPY --from=builder /app/target/release/rulecraft /app/rulecraft
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static
COPY --from=builder /app/migrations /app/migrations

# Create data directory
RUN mkdir -p /app/data && chown -R rulecraft:rulecraft /app

USER rulecraft

# Set environment
ENV DATABASE_URL=sqlite:./data/rulecraft.db
ENV PORT=3000

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["./rulecraft"]
