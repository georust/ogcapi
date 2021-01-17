# OGC API

[OGC API](https://ogcapi.ogc.org/) implementation in Rust based on [Tide](https://github.com/http-rs/tide) and [SQLx](https://github.com/launchbadge/sqlx)


## Prerequisites

- PostgreSQL
- Postgis
- OpenSSL

## Quick Start
```
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres

# Set the database URL
export DATABASE_URL=postgresql://postgres:postgres@localhost/ogcapi

# Setup the database 
sqlx database setup

# Run
cargo run
```