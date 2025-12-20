# Builder Stage
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /usr/src/vanity_crypto

# Copy manifests first to cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY .gitignore .

# Build release binary
RUN cargo build --release

# Runtime Stage (Secure, minimal image)
FROM debian:bookworm-slim

# Create non-root user
RUN useradd -ms /bin/bash vanity
USER vanity
WORKDIR /home/vanity

# Copy binary from builder
COPY --from=builder /usr/src/vanity_crypto/target/release/vanity_crypto /usr/local/bin/vanity_crypto

# Default command (Help)
CMD ["vanity_crypto", "--help"]
