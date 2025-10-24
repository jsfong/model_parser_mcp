FROM rust:1-alpine as builder
# Install build dependencies including OpenSSL
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig
RUN rustup default nightly
WORKDIR /app
COPY .sqlx ./.sqlx
COPY src ./src
COPY Cargo.toml Cargo.lock ./

# Install sqlx-cli
RUN cargo install sqlx-cli

# Enable offline mode for sqlx
ENV SQLX_OFFLINE=true

# Save queries
RUN cargo sqlx prepare --check

RUN cargo build --release

FROM gcr.io/distroless/static:nonroot
COPY --from=builder /app/target/release/model-parser-mcp /usr/local/bin/
ENTRYPOINT ["model-parser-mcp"]