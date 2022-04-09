# OGC API

[OGC API](https://ogcapi.ogc.org/) implementation in Rust leveraging [axum](https://github.com/tokio-rs/axum) and [SQLx](https://github.com/launchbadge/sqlx)

## Prerequisites

- Rust
- Docker & Docker Compose
- GDAL

## Quick Start

```
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres,rustls

# Setup the database
docker-compose up

# Create the schemas
sqlx database setup --source ogcapi-drivers/migrations

# Import some data
cargo run -p ogcapi_cli -- import ogcapi-cli/data/ne_10m_admin_0_countries.geojson --collection countries

# Serve 
cargo run -p ogcapi_cli -- serve
```
