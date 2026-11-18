mod args;
pub use args::{Args, Commands};

// #[cfg(feature = "stac")]
// mod asset;
#[cfg(feature = "geojson")]
pub mod geojson;
#[cfg(all(feature = "ogr", feature = "postgres"))]
pub mod ogr;

use ogcapi::types::common::Bbox;

/// `bbox` rounded outward to seven decimals, so that features on its edges
/// stay within it as it is transformed to another CRS and back.
#[must_use]
pub fn rounded_outward(bbox: &Bbox) -> Bbox {
    let down = |value: f64| (value * 1e7).floor() / 1e7;
    let up = |value: f64| (value * 1e7).ceil() / 1e7;
    match *bbox {
        Bbox::Bbox2D([west, south, east, north]) => {
            Bbox::Bbox2D([down(west), down(south), up(east), up(north)])
        }
        Bbox::Bbox3D([west, south, bottom, east, north, top]) => Bbox::Bbox3D([
            down(west),
            down(south),
            down(bottom),
            up(east),
            up(north),
            up(top),
        ]),
    }
}

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
