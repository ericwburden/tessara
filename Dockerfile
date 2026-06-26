# syntax=docker/dockerfile:1.7

FROM rust:1.94-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos --locked

WORKDIR /app
ARG APP_CACHE_BUST=dev
RUN echo "$APP_CACHE_BUST" >/tmp/app-cache-bust
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo leptos build --release --split \
    && cp /app/target/release/tessara-api /tmp/tessara-api \
    && cp -r /app/target/site /tmp/site

FROM debian:trixie-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /tmp/tessara-api /usr/local/bin/tessara-api
COPY --from=builder /tmp/site /app/site
COPY --from=builder /app/crates/tessara-api/migrations /app/migrations

ENV LEPTOS_SITE_ROOT=/app/site
ENV LEPTOS_SITE_PKG_DIR=pkg
ENV TESSARA_MIGRATIONS_DIR=/app/migrations

EXPOSE 8080
CMD ["tessara-api"]
