# Multi-stage build for ht-mcp-rust
FROM rust:1.85-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libfontconfig1-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY ht-core/Cargo.toml ./ht-core/

# Create dummy source files to cache dependencies
RUN mkdir -p src ht-core/src/api && \
    echo "fn main() {}" > src/main.rs && \
    echo "// dummy" > src/lib.rs && \
    echo "pub mod api; pub mod cli; pub mod command; pub mod locale; pub mod nbio; pub mod pty; pub mod session;" > ht-core/src/lib.rs && \
    echo "pub mod http; pub mod stdio;" > ht-core/src/api/mod.rs && \
    echo "// dummy" > ht-core/src/api/http.rs && \
    echo "// dummy" > ht-core/src/api/stdio.rs && \
    echo "// dummy" > ht-core/src/cli.rs && \
    echo "// dummy" > ht-core/src/command.rs && \
    echo "// dummy" > ht-core/src/locale.rs && \
    echo "// dummy" > ht-core/src/nbio.rs && \
    echo "// dummy" > ht-core/src/pty.rs && \
    echo "// dummy" > ht-core/src/session.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && \
    rm -rf src ht-core/src target/release/deps/ht_*

# Copy real source code
COPY src ./src
COPY ht-core/src ./ht-core/src
COPY ht-core/assets ./ht-core/assets

# Build the actual binary
RUN cargo build --release

# Runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    fontconfig \
    fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false ht-user

# Copy binary from builder
COPY --from=builder /app/target/release/ht-mcp /usr/local/bin/ht-mcp

# Set proper permissions
RUN chmod +x /usr/local/bin/ht-mcp

# Switch to non-root user
USER ht-user

# Set the entrypoint
ENTRYPOINT ["ht-mcp"]
CMD ["--help"]