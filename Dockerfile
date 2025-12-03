# Multi-stage Dockerfile for MD-Book
# This Dockerfile can be used with Fly.io, Railway, or any container platform

# Build stage
FROM rust:1.70 AS builder

WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY benches ./benches
COPY tests ./tests

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/md-book /usr/local/bin/md-book

# Copy templates and static assets
COPY src/templates ./templates

# Copy test input for demo (replace with your content)
COPY test_input ./test_input

# Create output directory
RUN mkdir -p dist

# Build the site at container startup
RUN md-book -i test_input -o dist

# Expose port (configurable via PORT env var)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:${PORT:-8080}/ || exit 1

# Start the server
CMD ["sh", "-c", "md-book -i test_input -o dist --serve --port ${PORT:-8080}"]
