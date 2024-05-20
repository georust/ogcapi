# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- This change log.
- Badges for `docs.rs` and `crates.io`.
- Categories and keywords.

### Changed

- Use top level `README.md` for `ogcapi` crate.
- Rework features to represent modules and standards in an orthogonal fashion.

## [0.2.0] - 2024-05-19

### Added
* Propagated `stac` feature to `ogcapi-client` by [@metasim](https://github.com/metasim) in (#11).
* Updated client `README.md` to work with latest APIs by [@metasim](https://github.com/metasim) in (#12).
* Updated workspace manifests to use relative paths to sibling packages by [@metasim](https://github.com/metasim) in (#14).
* Swap println with log::debug. by [@metasim](https://github.com/metasim) in (#17).
* Changes for usage in tile-grid by [@pka](https://github.com/pka) in (#18).


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


[unreleased]: https://github.com/georust/ogcapi/compare/v1.1.1...HEAD
[0.2.0]: https://github.com/georust/ogcapi/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/georust/ogcapi/releases/tag/v0.1.0