# syntax=docker/dockerfile:1.4
FROM switchboardlabs/sgx-function AS builder

# Install cargo-strip and use cache if available
RUN cargo install cargo-strip

WORKDIR /app
COPY . .

# Build the release binary
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
    cargo build --release && \
    cargo strip && \
    mv /app/target/release/app /app

