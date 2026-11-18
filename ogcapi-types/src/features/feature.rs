#[cfg(feature = "stac")]
use std::collections::HashMap;
use std::fmt::Display;

use crate::common::Bbox;

#[cfg(feature = "movingfeatures")]
use crate::movingfeatures::{
    crs::Crs, temporal_geometry::TemporalGeometry, temporal_properties::TemporalProperties,
    trs::Trs,
};

#[cfg(feature = "movingfeatures")]
use chrono::{DateTime, Utc};

use geojson::{Geometry, GeometryValue, Position};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::common::Link;

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum Type {
    #[default]
    Feature,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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

/// Geometry schema: a `GeoJSON` geometry, or `null` (RFC 7946, section 3.2).
///
/// # Panics
///
/// Panics if the included JSON is not valid. This should never happen, as the JSON is static and known to be valid.
#[cfg(feature = "utoipa")]
#[must_use]
pub fn geometry() -> utoipa::openapi::Schema {
    use utoipa::openapi::schema::{Object, OneOfBuilder, Schema, Type};

    let geometry: Schema = serde_json::from_str(include_str!("../../assets/schema/Geometry.json"))
        .expect("to be valid JSON");
    OneOfBuilder::new()
        .item(geometry)
        .item(Object::builder().schema_type(Type::Null))
        .into()
}

/// Abstraction of real world phenomena (ISO 19101-1:2014)
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Feature {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<FeatureId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "utoipa", schema(inline = true))]
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
    #[cfg_attr(feature = "utoipa", schema(value_type = Object))]
    pub feature_type: Option<jsonfg::FeatureType>,
    /// Link to the schema of the feature's `properties` (JSON-FG).
    #[cfg(feature = "json-fg")]
    #[serde(
        rename = "featureSchema",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "utoipa", schema(value_type = Object))]
    pub feature_schema: Option<jsonfg::FeatureSchema>,
    /// Coordinate reference system of `place` (JSON-FG).
    #[cfg(feature = "json-fg")]
    #[serde(
        rename = "coordRefSys",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "utoipa", schema(value_type = Object))]
    pub coord_ref_sys: Option<jsonfg::CoordRefSys>,
    /// The geometry in a non-WGS84 CRS (JSON-FG), which may use solids or curves. When
    /// set, the `GeoJSON` `geometry` member is `null`.
    #[cfg(feature = "json-fg")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "utoipa", schema(value_type = Object))]
    pub place: Option<jsonfg::Geometry>,
    /// The `GeoJSON` (WGS 84) geometry, or `null` for a feature without one (RFC 7946,
    /// section 3.2), or under `json-fg` when the geometry is carried in `place`.
    #[serde(default)]
    #[cfg_attr(feature = "utoipa", schema(schema_with = geometry, required = true))]
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
    /// A feature of `geometry`, with nothing else set.
    #[must_use]
    pub fn new(geometry: Geometry) -> Self {
        Feature {
            geometry: Some(geometry),
            ..Default::default()
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

/// A `Feature` without geometry and properties. Lets callers use `..Default::default()`.
impl Default for Feature {
    fn default() -> Self {
        Feature {
            id: None,
            collection: None,
            r#type: Type::Feature,
            properties: None,
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
    /// [`coord_ref_sys`](Feature::coord_ref_sys), and the `GeoJSON` `geometry` member is set
    /// to `null`; WGS 84 (`crs.is_wgs84()`, i.e. CRS84/CRS84h) is kept in `geometry`.
    /// Top-level `conformsTo` is set. Coordinates are not reprojected.
    ///
    /// This is for a single-feature document. For features inside a collection, the
    /// collection carries `conformsTo`/`coordRefSys` — see
    /// [`FeatureCollection::into_json_fg`](crate::features::FeatureCollection::into_json_fg).
    #[must_use]
    pub fn into_json_fg(self, crs: &jsonfg::CoordRefSys) -> Self {
        self.into_json_fg_scoped(crs, true)
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

impl Bbox {
    /// The box of `geometry`'s positions, those of a collection's members
    /// too; `None` for a geometry of none.
    #[must_use]
    pub fn of_geometry(geometry: &Geometry) -> Option<Bbox> {
        Bbox::extent(positions(geometry).map(Position::as_slice))
    }
}

/// The positions of `geometry`, those of a collection's members too.
fn positions(geometry: &Geometry) -> Box<dyn Iterator<Item = &Position> + '_> {
    match &geometry.value {
        GeometryValue::Point { coordinates } => Box::new(std::iter::once(coordinates)),
        GeometryValue::MultiPoint { coordinates } | GeometryValue::LineString { coordinates } => {
            Box::new(coordinates.iter())
        }
        GeometryValue::MultiLineString { coordinates } | GeometryValue::Polygon { coordinates } => {
            Box::new(coordinates.iter().flatten())
        }
        GeometryValue::MultiPolygon { coordinates } => {
            Box::new(coordinates.iter().flatten().flatten())
        }
        GeometryValue::GeometryCollection { geometries } => {
            Box::new(geometries.iter().flat_map(positions))
        }
    }
}

#[cfg(test)]
mod tests {
    use geojson::Geometry;

    use super::{Bbox, positions};

    #[test]
    fn positions_of_nested_collections() {
        let geometry: Geometry = serde_json::from_str(
            r#"{"type": "GeometryCollection", "geometries": [
                {"type": "Point", "coordinates": [1, 2]},
                {"type": "GeometryCollection", "geometries": [
                    {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [0, 1], [0, 0]]]}
                ]}
            ]}"#,
        )
        .unwrap();
        let all: Vec<&[f64]> = positions(&geometry)
            .map(geojson::Position::as_slice)
            .collect();
        assert_eq!(all.len(), 5);
        assert_eq!(all[0], [1.0, 2.0]);
        assert_eq!(all[4], [0.0, 0.0]);
        assert_eq!(
            Bbox::of_geometry(&geometry),
            Some(Bbox::Bbox2D([0.0, 0.0, 1.0, 2.0]))
        );
    }
}
