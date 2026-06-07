mod args;
pub use args::{Args, Commands};

#[cfg(feature = "geojson")]
pub mod geojson;
#[cfg(all(feature = "ogr", feature = "postgres"))]
pub mod ogr;

pub fn is_geojson_file(input: impl AsRef<std::path::Path>) -> bool {
    input
        .as_ref()
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_lowercase()
        .ends_with("geojson")
}
