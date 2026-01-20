# Build stage
FROM rust:1.83-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the application
# Touch main.rs to force rebuild of the binary
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/planka-mcp /usr/local/bin/planka-mcp

# Create a non-root user
RUN useradd -m -u 1000 planka && \
    chown planka:planka /usr/local/bin/planka-mcp

USER planka

# Set environment variables (can be overridden at runtime)
ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/planka-mcp"]
