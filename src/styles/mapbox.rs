use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A Mapbox style
#[derive(Serialize, Deserialize, Debug)]
struct Style {
    version: u32,
    name: String,
    sprite: String,
    glyphs: String,
    sources: HashMap<String, Source>,
    layers: Vec<Layer>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Layer {
    id: String,
    r#type: String, // Enum
    filter: Option<Value>,
    layout: Option<Value>,
    maxzoom: Option<u32>,
    mainzoom: Option<u32>,
    metadata: Option<Value>,
    paint: Option<Value>,
    source: Option<String>,
    source_layer: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Source {
    // name: String,
    r#type: SourceType,
    attribution: Option<String>,
    bounds: Option<Vec<f64>>,
    buffer: Option<u32>, // Only for geojson type
    #[serde(flatten)]
    cluster: Option<Cluster>, // Only for geojson type
    data: Option<String>, // Only for geojson type
    encoding: Option<String>, // Enum, only for raster-dem type
    filter: Option<String>, // Only for geojson type
    line_metric: Option<bool>, // Only for geojson type
    maxzoom: Option<u32>,
    minzoom: Option<u32>,
    promote_id: Option<Value>,
    scheme: Option<String>,
    tile_size: Option<u32>, // Only for raster* types
    tiles: Option<Vec<String>>,
    tolerance: Option<f64>, // Only for geojson type
    url: Option<String>,
    volatile: Option<bool>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
enum SourceType {
    Vector,
    Raster,
    RasterDem,
    Geojson,
    Image,
    Video,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Cluster {
    cluster: Option<bool>,
    cluster_max_zoom: Option<u32>,
    cluster_min_points: Option<u32>,
    cluster_properties: Option<Value>,
    cluster_radius: Option<u32>,
}
