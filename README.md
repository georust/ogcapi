# OGC API

[OGC API](https://ogcapi.ogc.org/) implementation in Rust leveraging [axum](https://github.com/tokio-rs/axum) and [SQLx](https://github.com/launchbadge/sqlx)


## Quick Start

This will take a while and use quite some disk space

```bash
# Setup the database
docker-compose up

# Import administrative bounaries
docker exec -ti ogcapi cargo run -- import ogcapi/data/ne_10m_admin_0_countries.geojson --collection countries
```

Open <http://localhost:8484/> were you will find the landing page.

## Developing

### Prerequisites

- Rust
- Docker & Docker Compose
- GDAL

```bash
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres,rustls
```

### Setup

```bash
# Setup the database
docker-compose up db

# Run tests
cargo test --workspace

# Import administrative bounaries
cargo run -- import ogcapi/data/ne_10m_admin_0_countries.geojson --collection countries

# Serve 
cargo run -- serve
```

## Teamengine

```bash
docker run --network host ogccite/ets-ogcapi-features10
# docker run --network host ogccite/ets-ogcapi-edr10
```
