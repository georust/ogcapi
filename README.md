# OGC API

[OGC API](https://ogcapi.ogc.org/) implementation in Rust based on [axum](https://github.com/tokio-rs/axum) and [SQLx](https://github.com/launchbadge/sqlx)


## Prerequisites

- PostgreSQL
- Postgis
- OpenSSL
- Gdal

```
# PostgreSQL with PostGIS
sudo apt-get install postgresql-13-postgis-3

# OpenSSL
sudo apt-get install pkg-config, libssl-dev

# Gdal
sudo apt-get install libgdal-dev gdal-bin
```

## Quick Start

```
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres,rustls

# Set the database URL
export DATABASE_URL=postgresql://postgres:postgres@localhost/ogcapi

# Setup the database
sqlx database setup

# Import some data
cargo run -- cargo run import data/ne_10m_admin_0_countries.geojson --collection countries

# Run cli serve help
cargo run -- serve
```