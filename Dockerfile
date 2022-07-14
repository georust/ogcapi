FROM rust:latest

RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libclang-dev \
    gdal-bin \
    libgdal-dev \
    postgresql-client

RUN rustup component add rustfmt clippy

WORKDIR /app

COPY ./ .
