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

#[cfg(feature = "coverages")]
mod coverages;
