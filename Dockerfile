# syntax=docker/dockerfile:1.7

FROM node:22-bookworm-slim AS styles

WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run tailwind:build

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
COPY --from=styles /app/target/site/pkg/tessara-web.css /tmp/tessara-web.css
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo leptos build --release --split \
    && cp /tmp/tessara-web.css /app/target/site/pkg/tessara-web.css \
    && cp /app/target/release/tessara-api /tmp/tessara-api \
    && cp -r /app/target/site /tmp/site

FROM debian:trixie-slim AS runtime

ARG TESSARA_SOURCE_COMMIT=unknown
ARG TESSARA_SOURCE_TREE=unknown
ARG TESSARA_SOURCE_DIRTY=unknown

LABEL org.opencontainers.image.revision="$TESSARA_SOURCE_COMMIT" \
      com.tessara.source-tree="$TESSARA_SOURCE_TREE" \
      com.tessara.source-dirty="$TESSARA_SOURCE_DIRTY" \
      com.tessara.build-profile="release"

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
