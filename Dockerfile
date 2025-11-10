# Multi-stage Dockerfile for Polis
FROM rust:1.91 as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libseccomp-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY polis-core/ polis-core/
COPY polis-runtime/ polis-runtime/
COPY polis-api/ polis-api/
COPY polis-cli/ polis-cli/
COPY polis-image/ polis-image/
COPY polis-network/ polis-network/
COPY polis-security/ polis-security/
COPY polis-storage/ polis-storage/
COPY polis-orchestrator/ polis-orchestrator/
COPY polis-monitor/ polis-monitor/
COPY polis-auth/ polis-auth/
COPY polis-benchmarks/ polis-benchmarks/
COPY polis-optimization/ polis-optimization/
COPY polis-sdk/ polis-sdk/
COPY polis-tests/ polis-tests/

# Build the project
RUN cargo build --release --bin polis-cli

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libseccomp2 \
    libssl3 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create polis user
RUN useradd -r -s /bin/false polis

# Set up directories
RUN mkdir -p /var/lib/polis /var/log/polis /etc/polis \
    && chown -R polis:polis /var/lib/polis /var/log/polis /etc/polis

# Copy binary from builder stage
COPY --from=builder /app/target/release/polis-cli /usr/local/bin/

# Copy configuration files
COPY docs/ /usr/share/doc/polis/
COPY examples/ /usr/share/doc/polis/examples/

# Set permissions
RUN chmod +x /usr/local/bin/polis-cli

# Expose ports
EXPOSE 8080 50051

# Switch to polis user
USER polis

# Set working directory
WORKDIR /var/lib/polis

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["polis-cli", "server", "start"]
