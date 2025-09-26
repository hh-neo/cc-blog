# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install dependencies
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/messageboard /usr/local/bin/messageboard
RUN chown appuser:appuser /usr/local/bin/messageboard

USER appuser

EXPOSE 8080

CMD ["messageboard"]