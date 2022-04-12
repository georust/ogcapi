FROM rust:latest

RUN apt-get update && apt-get install -y build-essential pkg-config libclang-dev gdal-bin libgdal-dev postgresql-client

RUN cargo install sqlx-cli --no-default-features --features postgres,rustls
RUN cargo install cargo-watch

WORKDIR /app

COPY ./ .
