# Build stage
FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY migrations ./migrations
COPY templates ./templates

# Copy source code
COPY src ./src

# Build the application in release mode
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/realworld-rust-fullstack /app/server

# Copy necessary runtime files
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/migrations ./migrations

# Expose the port
EXPOSE 8080

# Set environment defaults
ENV HOST=0.0.0.0
ENV PORT=8080

# Run the binary
CMD ["./server"]
