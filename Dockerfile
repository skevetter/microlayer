# Build stage
FROM rust:slim AS builder

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:trixie-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /build/target/release/picolayer /usr/local/bin/picolayer

# Make binary executable
RUN chmod +x /usr/local/bin/picolayer

# Set the binary as entrypoint
ENTRYPOINT ["/usr/local/bin/picolayer"]

# Default command shows help
CMD ["--help"]
