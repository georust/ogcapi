use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use utoipa::{IntoParams, ToSchema};

use crate::common::{Bbox, Datetime};

/// Search parameters for searching a SpatioTemporal Asset Catalog.
#[serde_with::serde_as]
#[derive(Serialize, Deserialize, IntoParams, Default, Debug)]
#[into_params(parameter_in = Query)]
pub struct SearchParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[param(value_type = usize, nullable = false, required = false)]
    pub limit: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[param(value_type = usize, nullable = false, required = false)]
    pub offset: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[param(value_type = Bbox, style = Form, explode = false, nullable = false)]
    pub bbox: Option<Bbox>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[param(value_type = String, nullable = false)]
    pub datetime: Option<Datetime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[param(schema_with = crate::features::geometry)]
    pub intersects: Option<Geometry>,
    #[serde(
        default,
        with = "serde_qs::helpers::comma_separated",
        skip_serializing_if = "Vec::is_empty"
    )]
    #[param(style = Form, explode = false, required = false)]
    pub ids: Vec<String>,
    #[serde(
        default,
        with = "serde_qs::helpers::comma_separated",
        skip_serializing_if = "Vec::is_empty"
    )]
    #[param(style = Form, explode = false, required = false)]
    pub collections: Vec<String>,
}

impl SearchParams {
    /// Create a new search parameter builder
    /// # Example:
    ///
    /// ```rust
    /// use ogcapi_types::stac::SearchParams;
    /// use ogcapi_types::common::Bbox;
    ///
    /// let bbox = Bbox::from([7.4513398, 46.92792859, 7.4513662, 46.9279467]);
    ///
    /// let params = SearchParams::new()
    ///     .with_collections(["communes"].as_slice())
    ///     .with_bbox(bbox);
    /// ```
    pub fn new() -> SearchParams {
        SearchParams::default()
    }

    /// Set the `bbox` property
    pub fn with_bbox(mut self, bbox: Bbox) -> Self {
        self.bbox = Some(bbox);
        self
    }

    /// Set the `datetime` property
    pub fn with_datetime(mut self, datetime: Datetime) -> Self {
        self.datetime = Some(datetime);
        self
    }

    /// Set the `intersects` property
    pub fn with_intersects(mut self, intersects: Geometry) -> Self {
        self.intersects = Some(intersects);
        self
    }

    /// Set the `ids` property
    pub fn with_ids<S, I>(mut self, ids: I) -> Self
    where
        S: std::fmt::Display,
        I: IntoIterator<Item = S>,
    {
        self.ids = ids.into_iter().map(|i| i.to_string()).collect();
        self
    }

    /// Set the `collections` property
    pub fn with_collections<S, I>(mut self, collections: I) -> Self
    where
        S: std::fmt::Display,
        I: IntoIterator<Item = S>,
    {
        self.collections = collections.into_iter().map(|c| c.to_string()).collect();
        self
    }
}

/// Search body for searching a SpatioTemporal Asset Catalog.
#[serde_with::serde_as]
#[derive(Serialize, Deserialize, ToSchema, Default, Debug)]
pub struct SearchBody {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub bbox: Option<Bbox>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = String)]
    pub datetime: Option<Datetime>,
    #[serde(default)]
    #[schema(schema_with = crate::features::geometry)]
    pub intersects: Option<Geometry>,
    #[serde(default)]
    pub ids: Vec<String>,
    #[serde(default)]
    pub collections: Vec<String>,
}

impl From<SearchBody> for SearchParams {
    fn from(body: SearchBody) -> Self {
        SearchParams {
            limit: body.limit,
            offset: body.offset,
            bbox: body.bbox,
            datetime: body.datetime,
            intersects: body.intersects,
            ids: body.ids,
            collections: body.collections,
        }
    }
}
