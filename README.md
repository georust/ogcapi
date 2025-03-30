# ogcapi

[![Documentation](https://docs.rs/ogcapi/badge.svg)](https://docs.rs/ogcapi)
[![Crate](https://img.shields.io/crates/v/ogcapi.svg)](https://crates.io/crates/ogcapi)

The `ogcapi` crate provides various [OGC API](https://ogcapi.ogc.org/) building blocks implemented in [Rust](https://www.rust-lang.org/).


## Project Outline

The code is organized in four modules, respectively crates:

| Module / Crate    | Description     |
| ----------------- | --------------- |
| `ogcapi-types`    | Types as defined in various OGC API standards as well as STAC with `serde` support. |
| `ogcapi-client`   | Client to access HTTP endpoints of OGC API services as well as STAC wrapping `reqwest` |
| `ogcapi-services` | Server implementation of various OGC API services based on `axum`. |
| `ogcapi-drivers`  | Drivers for different data provider backends, currently mainly PostgreSQL with PostGIS through `sqlx`. |

These modules are reexported within the `ogcapi` crate.

## Quick Start (Podman/Docker)

This will take a while and use quite some disk space

```bash
# Setup the database
podman compose up --build

# Import administrative bounaries
podman exec -ti ogcapi cargo run -p data-loader -- --input data/ne_110m_admin_0_countries.geojson --collection countries

# Run app
podman exec -ti ogcapi cargo run -p demo-service
```

Open <http://localhost:8484/> were you will find the `Landing Page`.

## Developing

### Prerequisites

- Rust
- Podman or Docker
- GDAL

```bash
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres,rustls
```

### Setup

```bash
# Run services
podman compose up db minio minio-mc -d

# Import administrative bounaries
cargo run -p data-loader -- --input data/ne_110m_admin_0_countries.geojson --collection countries

# Start service 
cargo run -p demo-service

# Run tests
cargo test --workspace --all-features

# Open Documentation
cargo doc --workspace --all-features --no-deps --open
```

### Format / Lint

```bash
# Format
cargo fmt

# Clippy
cargo clippy --workspace --all-features --examples --tests
```

### Teamengine

```bash
podman run --network host docker.io/ogccite/ets-ogcapi-features10
# podman run --network host docker.io/ogccite/ets-ogcapi-edr10
```

Navigate to <http://localhost:8080/teamengine/> to execute the test suite. For documentation and more info see <https://cite.opengeospatial.org/teamengine/about/ogcapi-features-1.0/1.0/site>.

## Example Project

Based on this project, a STAC enabled OGC API Features service has successfully been setup. You can find the code from the prove of concept [here](https://github.com/camptocamp/oapi-poc)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
