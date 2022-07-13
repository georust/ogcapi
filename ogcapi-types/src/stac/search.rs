use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::common::{Bbox, Datetime, ListParam};

/// Search parameters for searching a SpatioTemporal Asset Catalog.
#[serde_as]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SearchParams {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bbox: Option<Bbox>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub datetime: Option<Datetime>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub intersects: Option<Geometry>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub ids: Option<ListParam>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub collections: Option<ListParam>,
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
    pub fn with_ids(mut self, ids: impl Into<ListParam>) -> Self {
        self.ids = Some(ids.into());
        self
    }

    /// Set the `collections` property
    pub fn with_collections(mut self, collections: impl Into<ListParam>) -> Self {
        self.collections = Some(collections.into());
        self
    }
}

/// Search body for searching a SpatioTemporal Asset Catalog.
#[serde_as]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SearchBody {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bbox: Option<Bbox>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub datetime: Option<Datetime>,
    #[serde(default)]
    // #[serde_as(as = "Option<DisplayFromStr>")]
    pub intersects: Option<Geometry>,
    pub ids: Option<ListParam>,
    pub collections: Option<ListParam>,
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
