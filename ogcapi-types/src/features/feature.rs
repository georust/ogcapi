#[cfg(feature = "stac")]
use std::collections::HashMap;
use std::fmt::Display;

#[cfg(feature = "stac")]
use crate::common::Bbox;

#[cfg(feature = "movingfeatures")]
use crate::movingfeatures::{
    crs::Crs, temporal_geometry::TemporalGeometry, temporal_properties::TemporalProperties,
    trs::Trs,
};

#[cfg(feature = "movingfeatures")]
use chrono::{DateTime, Utc};

use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::{ToSchema, openapi::Schema};

use crate::common::Link;

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    #[default]
    Feature,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum FeatureId {
    String(String),
    Integer(u64),
}

impl Display for FeatureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureId::String(s) => f.write_str(s),
            FeatureId::Integer(i) => f.write_fmt(format_args!("{i}")),
        }
    }
}

/// Geometry schema.
pub fn geometry() -> Schema {
    serde_json::from_str(include_str!("../../assets/schema/Geometry.json")).unwrap()
}

/// Abstraction of real world phenomena (ISO 19101-1:2014)
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone, PartialEq)]
pub struct Feature {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<FeatureId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    #[serde(default)]
    #[schema(inline = true)]
    pub r#type: Type,
    #[serde(default)]
    pub properties: Option<Map<String, Value>>,
    /// Declared JSON-FG conformance classes. Set only on the top-level object.
    #[cfg(feature = "json-fg")]
    #[serde(
        rename = "conformsTo",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub conforms_to: Option<Vec<String>>,
    /// JSON-FG feature classifier(s).
    #[cfg(feature = "json-fg")]
    #[serde(
        rename = "featureType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[schema(value_type = Object)]
    pub feature_type: Option<jsonfg::FeatureType>,
    /// Link to the schema of the feature's `properties` (JSON-FG).
    #[cfg(feature = "json-fg")]
    #[serde(
        rename = "featureSchema",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[schema(value_type = Object)]
    pub feature_schema: Option<jsonfg::FeatureSchema>,
    /// Coordinate reference system of `place` (JSON-FG).
    #[cfg(feature = "json-fg")]
    #[serde(
        rename = "coordRefSys",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[schema(value_type = Object)]
    pub coord_ref_sys: Option<jsonfg::CoordRefSys>,
    /// The geometry in a non-WGS84 CRS (JSON-FG), which may use solids or curves. When
    /// set, the GeoJSON `geometry` member is `null`.
    #[cfg(feature = "json-fg")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub place: Option<jsonfg::Geometry>,
    /// The GeoJSON (WGS 84) geometry.
    #[cfg(not(feature = "json-fg"))]
    #[schema(schema_with = geometry)]
    pub geometry: Geometry,
    /// The GeoJSON (WGS 84) geometry, or `null`. Nullable under `json-fg` so it can be
    /// `null` when the geometry is carried in [`place`](Feature::place).
    #[cfg(feature = "json-fg")]
    #[serde(default)]
    #[schema(schema_with = geometry)]
    pub geometry: Option<Geometry>,
    /// Bounding Box of the asset represented by this Item, formatted according to RFC 7946, section 5.
    #[cfg(feature = "stac")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Bbox>,
    #[serde(default)]
    pub links: Vec<Link>,
    /// The STAC version the Item implements.
    #[cfg(feature = "stac")]
    #[serde(default = "crate::stac::stac_version")]
    pub stac_version: String,
    /// A list of extensions the Item implements.
    #[cfg(feature = "stac")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stac_extensions: Vec<String>,
    /// Dictionary of asset objects that can be downloaded, each with a unique key.
    #[cfg(feature = "stac")]
    #[serde(default)]
    pub assets: HashMap<String, crate::stac::Asset>,
    #[cfg(feature = "movingfeatures")]
    #[serde(
        default,
        serialize_with = "crate::common::serialize_interval",
        skip_serializing_if = "Vec::is_empty"
    )]
    /// Life span information for the moving feature.
    /// See [MF-Json 7.2.3 LifeSpan](https://docs.ogc.org/is/19-045r3/19-045r3.html#time)
    pub time: Vec<[Option<DateTime<Utc>>; 2]>,
    #[cfg(feature = "movingfeatures")]
    #[serde(default)]
    pub crs: Crs,
    #[cfg(feature = "movingfeatures")]
    #[serde(default)]
    pub trs: Trs,
    #[cfg(feature = "movingfeatures")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "temporalGeometry"
    )]
    pub temporal_geometry: Option<TemporalGeometry>,
    #[cfg(feature = "movingfeatures")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "temporalProperties"
    )]
    pub temporal_properties: Option<TemporalProperties>,
}

impl Feature {
    pub fn new(geometry: Geometry) -> Self {
        Feature {
            id: Default::default(),
            collection: Default::default(),
            r#type: Default::default(),
            properties: Default::default(),
            #[cfg(feature = "json-fg")]
            conforms_to: None,
            #[cfg(feature = "json-fg")]
            feature_type: None,
            #[cfg(feature = "json-fg")]
            feature_schema: None,
            #[cfg(feature = "json-fg")]
            coord_ref_sys: None,
            #[cfg(feature = "json-fg")]
            place: None,
            #[cfg(not(feature = "json-fg"))]
            geometry,
            #[cfg(feature = "json-fg")]
            geometry: Some(geometry),
            #[cfg(feature = "stac")]
            bbox: Default::default(),
            links: Default::default(),
            #[cfg(feature = "stac")]
            stac_version: crate::stac::stac_version(),
            #[cfg(feature = "stac")]
            stac_extensions: Default::default(),
            #[cfg(feature = "stac")]
            assets: Default::default(),
            #[cfg(feature = "movingfeatures")]
            time: Default::default(),
            #[cfg(feature = "movingfeatures")]
            crs: Default::default(),
            #[cfg(feature = "movingfeatures")]
            trs: Default::default(),
            #[cfg(feature = "movingfeatures")]
            temporal_geometry: Default::default(),
            #[cfg(feature = "movingfeatures")]
            temporal_properties: Default::default(),
        }
    }

    pub fn append_properties(&mut self, mut other: Map<String, Value>) {
        if let Some(properties) = self.properties.as_mut() {
            properties.append(&mut other);
        } else {
            self.properties = Some(other);
        }
    }
}

/// With `json-fg`, `geometry` is nullable, so a `Feature` has a natural empty default
/// (null geometry, no properties). Lets callers use `..Default::default()`.
#[cfg(feature = "json-fg")]
impl Default for Feature {
    fn default() -> Self {
        Feature {
            id: None,
            collection: None,
            r#type: Type::Feature,
            properties: None,
            conforms_to: None,
            feature_type: None,
            feature_schema: None,
            coord_ref_sys: None,
            place: None,
            geometry: None,
            #[cfg(feature = "stac")]
            bbox: None,
            links: Vec::new(),
            #[cfg(feature = "stac")]
            stac_version: crate::stac::stac_version(),
            #[cfg(feature = "stac")]
            stac_extensions: Vec::new(),
            #[cfg(feature = "stac")]
            assets: Default::default(),
            #[cfg(feature = "movingfeatures")]
            time: Vec::new(),
            #[cfg(feature = "movingfeatures")]
            crs: Default::default(),
            #[cfg(feature = "movingfeatures")]
            trs: Default::default(),
            #[cfg(feature = "movingfeatures")]
            temporal_geometry: None,
            #[cfg(feature = "movingfeatures")]
            temporal_properties: None,
        }
    }
}

/// JSON-FG output conversion.
#[cfg(feature = "json-fg")]
impl Feature {
    /// Rewrite this feature for JSON-FG output given the geometry's `crs`.
    ///
    /// Geometry in a non-WGS 84 CRS is moved into [`place`](Feature::place) with a
    /// [`coord_ref_sys`](Feature::coord_ref_sys), and the GeoJSON `geometry` member is set
    /// to `null`; WGS 84 (`crs.is_wgs84()`, i.e. CRS84/CRS84h) is kept in `geometry`.
    /// Top-level `conformsTo` is set. Coordinates are not reprojected.
    ///
    /// This is for a single-feature document. For features inside a collection, the
    /// collection carries `conformsTo`/`coordRefSys` — see
    /// [`FeatureCollection::into_json_fg`](crate::features::FeatureCollection::into_json_fg).
    pub fn into_json_fg(self, crs: jsonfg::CoordRefSys) -> Self {
        self.into_json_fg_scoped(&crs, true)
    }

    /// As [`into_json_fg`](Feature::into_json_fg); `top_level` gates the feature-level
    /// `conformsTo`/`coordRefSys` (omitted for features nested in a collection).
    pub(crate) fn into_json_fg_scoped(
        mut self,
        crs: &jsonfg::CoordRefSys,
        top_level: bool,
    ) -> Self {
        let geometry = self.geometry.take();
        if crs.is_wgs84() {
            self.geometry = geometry;
        } else {
            self.place = geometry.map(jsonfg::Geometry::from);
            self.geometry = None;
            if top_level {
                self.coord_ref_sys = Some(crs.clone());
            }
        }
        if top_level {
            self.conforms_to = Some(vec![jsonfg::conformance::CORE.to_owned()]);
        }
        self
    }
}
