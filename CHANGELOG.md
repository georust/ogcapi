# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Dynamic OpenAPI definition extraction.
- Async OGC API - Processes execution (jobs).
- Multipart/related response support for raw OGC API - Processes results with multiple outputs.
- Echo process for testing.

### Fixed

- Respect process execution `response` parameter
- Service URL for OGC API - Features

### Changed

- Make features opt-out rather than opt-in for released standards.
- Allow integers for feature id.
- Build documentation for all features.
- Output type for OGC API - Processes trait (execute).
- Changed fields to status database model for OGC API - Processes.

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
