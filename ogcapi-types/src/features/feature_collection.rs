use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::Link;

#[cfg(feature = "movingfeatures")]
use crate::common::Bbox;

use super::Feature;

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    #[default]
    FeatureCollection,
}

/// A set of Features from a dataset
#[derive(Serialize, Deserialize, ToSchema, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCollection {
    #[serde(default)]
    #[schema(inline = true)]
    pub r#type: Type,
    pub features: Vec<Feature>,
    #[serde(default)]
    pub links: Vec<Link>,
    /// This property indicates the time and date when the response was generated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_stamp: Option<String>,
    /// The number of features of the feature type that match the selection
    /// parameters like `bbox`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_matched: Option<u64>,
    /// The number of features in the feature collection.
    ///
    /// A server may omit this information in a response, if the information
    /// about the number of features is not known or difficult to compute.
    ///
    /// If the value is provided, the value shall be identical to the number
    /// of items in the "features" array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_returned: Option<u64>,
    /// Declared JSON-FG conformance classes (top-level object only).
    #[cfg(feature = "json-fg")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conforms_to: Option<Vec<String>>,
    /// Default coordinate reference system for the features' `place` geometries (JSON-FG).
    #[cfg(feature = "json-fg")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub coord_ref_sys: Option<jsonfg::CoordRefSys>,
    /// JSON-FG feature classifier(s) shared by the collection.
    #[cfg(feature = "json-fg")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub feature_type: Option<jsonfg::FeatureType>,
    /// Dimension (0–3) of the geometries in this collection, if uniform (JSON-FG).
    #[cfg(feature = "json-fg")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry_dimension: Option<u8>,
    #[cfg(feature = "movingfeatures")]
    #[serde(default)]
    pub crs: crate::movingfeatures::crs::Crs,
    #[cfg(feature = "movingfeatures")]
    #[serde(default)]
    pub trs: crate::movingfeatures::trs::Trs,
    #[cfg(feature = "movingfeatures")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Bbox>,
}

impl FeatureCollection {
    pub fn new(features: Vec<Feature>) -> Self {
        let number_returned = features.len();
        FeatureCollection {
            features,
            time_stamp: Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
            number_returned: Some(number_returned as u64),
            ..Default::default()
        }
    }
}

/// JSON-FG output conversion.
#[cfg(feature = "json-fg")]
impl FeatureCollection {
    /// Rewrite this collection for JSON-FG output given the geometries' `crs`: set the
    /// collection-level `conformsTo` and `coordRefSys`, and move each feature's geometry
    /// into `place` with `geometry` set to `null`. Features inherit the collection CRS, so
    /// they carry neither their own `coordRefSys` nor `conformsTo`. WGS 84
    /// (`crs.is_wgs84()`) is kept in `geometry`. Coordinates are not reprojected.
    pub fn into_json_fg(mut self, crs: jsonfg::CoordRefSys) -> Self {
        self.conforms_to = Some(vec![jsonfg::conformance::CORE.to_owned()]);
        if !crs.is_wgs84() {
            self.coord_ref_sys = Some(crs.clone());
        }
        self.features = self
            .features
            .into_iter()
            .map(|f| f.into_json_fg_scoped(&crs, false))
            .collect();
        self
    }
}
