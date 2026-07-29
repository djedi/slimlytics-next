# syntax=docker/dockerfile:1.7
FROM rust:1.97-bookworm AS builder
WORKDIR /src
COPY backend/Cargo.toml backend/Cargo.lock* ./backend/
COPY backend/src ./backend/src
COPY migrations ./migrations
WORKDIR /src/backend
RUN cargo build --locked --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libssl3 \
    && rm -rf /var/lib/apt/lists/*
RUN useradd --system --uid 10001 --create-home slimlytics
COPY --from=builder /src/backend/target/release/slimlytics-backend /usr/local/bin/slimlytics
COPY migrations /app/migrations
WORKDIR /app
USER slimlytics
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/slimlytics"]
