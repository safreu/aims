FROM rust:bookworm AS builder

WORKDIR /app

ENV SQLX_OFFLINE=true

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY .sqlx .sqlx

COPY backend/Cargo.toml backend/Cargo.toml
COPY scanner/Cargo.toml scanner/Cargo.toml
COPY updater/Cargo.toml updater/Cargo.toml

RUN mkdir -p backend/src scanner/src updater/src \
    && echo "fn main() {}" > backend/src/main.rs \
    && echo "fn main() {}" > scanner/src/main.rs \
    && echo "fn main() {}" > updater/src/main.rs

RUN cargo build --release -p backend

RUN rm -rf target/release/deps/backend* \
    target/release/backend \
    target/release/.fingerprint/backend-*


COPY backend/src backend/src
COPY backend/migrations backend/migrations

RUN cargo build --release -p backend

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/backend /usr/local/bin/aims-backend

EXPOSE 3000

CMD [ "aims-backend" ]