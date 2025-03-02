#[cfg(feature = "greeter")]
pub mod greeter;

#[cfg(feature = "gdal-loader")]
pub mod gdal_loader;

#[cfg(feature = "geojson-loader")]
pub mod geojson_loader;

mod processor;
pub use processor::*;
