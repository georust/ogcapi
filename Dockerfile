FROM rust:latest

ARG CARGO_WATCH

RUN apt-get update && apt-get install -y build-essential pkg-config libclang-dev gdal-bin libgdal-dev postgresql-client

RUN rustup component add rustfmt clippy

RUN cargo install sqlx-cli --no-default-features --features postgres,rustls

RUN if [ "$CARGO_WATCH" ] ; then cargo install cargo-binstall && cargo binstall cargo-watch --no-confirm ; else echo "Skipping installation of cargo watch"; fi

WORKDIR /app

COPY ./ .
