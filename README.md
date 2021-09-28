# OGC API

[OGC API](https://ogcapi.ogc.org/) implementation in Rust based on [Tide](https://github.com/http-rs/tide) and [SQLx](https://github.com/launchbadge/sqlx)


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
cargo install sqlx-cli --no-default-features --features postgres

# Set the database URL
export DATABASE_URL=postgresql://postgres:postgres@localhost/ogcapi

# Setup the database
sqlx database setup

# Import some data
cargo run -- import https://d2ad6b4ur7yvpq.cloudfront.net/naturalearth-3.3.0/ne_110m_admin_1_states_provinces_shp.geojson

# Run cli serve help
cargo run -- serve
```