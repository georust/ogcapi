# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Async client transactions (CRUD) suport for JSON resources.
- `wasm32-unknown-unknown` support for `ogcapi-client`.
- `Client::new_with` and `BlockingClient::new_with` to bring a custom `reqwest` client.
- `Client::items_with_query` and `BlockingClient::items_with_query` for server-side filtering by bbox, datetime, etc.
- Dynamic OpenAPI definition extraction.
- Types for `OGCAPI - Moving Features`
- Async OGC API - Processes execution (jobs).
- Multipart/related response support for raw OGC API - Processes results with multiple outputs.
- Echo process for testing.
- Dedicate `cite-service` for OGC Cite testsuite in CI.
- Make public base url configurable via `PUBLIC_URL` environment variable.
- Allow modifying the spawn function (`with_spawn_fn`) in `AppState`, which is used for OGC API - Processes execution, so that it can be adapted, e.g., for applying scopes.
- Allow modifying the router and OpenAPI definition in the `Service` (`get_router_mut`), e.g., for adding additional paths or for changing the info fields in the OpenAPI definition.
- Allow modifying the middleware stack in the `Service` (`get_middleware_stack_mut`), e.g., for adding additional middleware or replacing the default ones.
- Optional JSON-FG (OGC Features and Geometries JSON) support in `ogcapi-types` behind a `json-fg` feature: JSON-FG members on `Feature` and `FeatureCollection` (`place`, `coordRefSys`, `conformsTo`, ...), `into_json_fg` conversions, the `application/vnd.ogc.fg+json` media type, and a re-export of the [`jsonfg`](https://crates.io/crates/jsonfg) crate. By [@georgeboot](https://github.com/georgeboot).

### Fixed

- `Extent.spatial` and `Extent.temporal` are `Option` again, matching the OGC spec and the published 0.3.0 crate. Regression from #26.
- Respect process execution `response` parameter.
- Service URL for OGC API - Features.
- Minor issues with OGC API - Features conformance.
- Changed enum order when deserializing `processes` inputs, so that the integers would not be deserialized as floats.
- The description fields were missing in the process summary of the OGC API Processes implementation, so they were added.
- Fixed serialization of `TileMatrixSetId` in OGC API - Tiles.
- `Collection.keywords` is now gated behind the standards that define it (new `records` feature flag, `stac`, `edr`), so builds without those standards no longer fail to deserialize collections whose `keywords` use a nonconforming extension shape. By [@georgeboot](https://github.com/georgeboot).

### Changed

- Drop `osm` example.
- Bump dependencies.
- Typed `z` edr query parameter.
- BREAKING: `ogcapi-client`: `Client` is now async by default. The previous blocking `Client` is available as `BlockingClient` behind the `blocking` feature flag.
- Make features opt-out rather than opt-in for released standards.
- Allow integers for feature id.
- Build documentation for all features.
- Output type for OGC API - Processes trait (execute).
- Changed fields to status database model for OGC API - Processes.
- Consolidate API definition for OGC cite validation.
- Decouple drivers from app state.
- Set default item type of collection as `feature`.
- Define numeric feature id as `u64`.
- Remove default Crs implementation.

## [0.3.0] - 2025-04-05

### Added

- This changelog.
- Badges for `docs.rs` and `crates.io`.
- Categories and keywords.

### Fixed

- Temporal extent serialization by [@jacovdbergh](https://github.com/jacovdbergh).

### Changed

- Update to 2024 edition.
- Update dependencies.
- Use top level `README.md` for `ogcapi` crate.
- Rework features to represent modules and standards in an orthogonal fashion.
- Convert CLI to example crates.
- Align `OGCAPI - Processes` with released core standard.
- Refactor `processes`.

## [0.2.0] - 2024-05-19

### Added
* Propagated `stac` feature to `ogcapi-client` by [@metasim](https://github.com/metasim) in [#11](https://github.com/georust/ogcapi/pull/11).
* Updated client `README.md` to work with latest APIs by [@metasim](https://github.com/metasim) in [#12](https://github.com/georust/ogcapi/pull/12).
* Updated workspace manifests to use relative paths to sibling packages by [@metasim](https://github.com/metasim) in [#14](https://github.com/georust/ogcapi/pull/14)
* Swap println with log::debug. by [@metasim](https://github.com/metasim) in [#17](https://github.com/georust/ogcapi/pull/17).
* Changes for usage in tile-grid by [@pka](https://github.com/pka) in [#18](https://github.com/georust/ogcapi/pull/18).


### Changed
- Various additions and fixes for types
- Reworked database schema
- Updated dependencies

## [0.1.0] - 2022-08-05

### Added

- Types for various OGC API standards and the SpatioTemporal Asset Catalog (STAC) specification
- SpatioTemporal Asset Catalog (STAC) features
- Server & Client implementation
- Add async traits for drivers (data sources)
- GDAL and Geojson importer
- Proof of concept for STAC / OGC API - Features service
- License as MIT/Apache-2.0
- Basic CI


[unreleased]: https://github.com/georust/ogcapi/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/georust/ogcapi/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/georust/ogcapi/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/georust/ogcapi/releases/tag/v0.1.0
