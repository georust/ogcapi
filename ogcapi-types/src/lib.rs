#![doc = include_str!("../README.md")]

/// Types specified in the `OGC API - Common` standard.
#[cfg(feature = "common")]
pub mod common;
/// Types specified in the `OGC API - Environmental Data Retrieval` standard.
#[cfg(feature = "edr")]
pub mod edr;
/// Types specified in the `OGC API - Features` standard.
#[cfg(feature = "features")]
pub mod features;
/// Types specified in the `OGC API - Moving Features` standard.
#[cfg(feature = "movingfeatures")]
pub mod movingfeatures;
/// Types specified in the `OGC API - Processes` standard.
#[cfg(feature = "processes")]
pub mod processes;
/// Types from the `SpatioTemporal Asset Catalog` specfication.
#[cfg(feature = "stac")]
pub mod stac;
/// Types specified in the `OGC API - Styles` standard.
#[cfg(feature = "styles")]
pub mod styles;
/// Types specified in the `OGC API - Tiles` standard.
#[cfg(feature = "tiles")]
pub mod tiles;

/// Types for `OGC Features and Geometries JSON` (JSON-FG), re-exported from the
/// [`jsonfg`] crate. Use these — most importantly [`jsonfg::Geometry`] (which, unlike
/// [`geojson::Geometry`], can represent solids and curves) and [`jsonfg::Feature`] — to
/// produce JSON-FG responses. Enabled by the `json-fg` feature.
#[cfg(feature = "json-fg")]
pub use jsonfg;

// #[cfg(feature = "coverages")]
// mod coverages;
