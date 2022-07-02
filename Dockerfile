FROM rust:latest

RUN apt-get update && apt-get install -y \
        build-essential \
        pkg-config \
        libclang-dev \
        gdal-bin \
        libgdal-dev \
        postgresql-client

RUN rustup component add rustfmt clippy

ARG CARGO_WATCH
RUN if [ $CARGO_WATCH = true ] ; \
    then cargo install cargo-watch ; \
    else echo "Skip installing `cargo-watch`"; \
    fi

WORKDIR /app

COPY ./ .
