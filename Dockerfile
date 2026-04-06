FROM rust:1.94-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release -p tessara-api

FROM debian:trixie-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates python3-minimal \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/tessara-api /usr/local/bin/tessara-api

EXPOSE 8080
CMD ["tessara-api"]
