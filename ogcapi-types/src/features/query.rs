use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use utoipa::{IntoParams, ToSchema};

use crate::common::{Bbox, Crs, Datetime};

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, IntoParams, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Query {
    /// The optional limit parameter limits the number of items that are
    /// presented in the response document.
    ///
    /// Only items are counted that are on the first level of the  collection
    /// in the response document. Nested objects contained
    /// within the explicitly requested items shall not be counted.
    #[param(nullable = false)]
    pub limit: Option<usize>,
    #[param(nullable = false)]
    pub offset: Option<usize>,
    /// Only features that have a geometry that intersects the bounding box
    /// are selected. The bounding box is provided as four or six numbers,
    /// depending on whether the coordinate reference system includes a
    /// vertical axis (height or depth):
    ///
    /// * Lower left corner, coordinate axis 1
    /// * Lower left corner, coordinate axis 2
    /// * Minimum value, coordinate axis 3 (optional)
    /// * Upper right corner, coordinate axis 1
    /// * Upper right corner, coordinate axis 2
    /// * Maximum value, coordinate axis 3 (optional)
    ///
    /// If the value consists of four numbers, the coordinate reference system
    /// is WGS 84 longitude/latitude (http://www.opengis.net/def/crs/OGC/1.3/CRS84)
    /// unless a different coordinate reference system is specified in the
    /// parameter `bbox-crs`.
    ///
    /// If the value consists of six numbers, the coordinate reference system
    /// is WGS 84 longitude/latitude/ellipsoidal height (http://www.opengis.net/def/crs/OGC/0/CRS84h)
    /// unless a different coordinate reference system is specified in the
    /// parameter `bbox-crs`.
    ///
    /// The query parameter bbox-crs is specified in OGC API - Features - Part 2:
    /// Coordinate Reference Systems by Reference.
    ///
    /// For WGS 84 longitude/latitude the values are in most cases the sequence
    /// of minimum longitude, minimum latitude, maximum longitude and maximum
    /// latitude. However, in cases where the box spans the antimeridian the
    /// first value (west-most box edge) is larger than the third value
    /// (east-most box edge).
    ///
    /// If the vertical axis is included, the third and the sixth number are
    /// the bottom and the top of the 3-dimensional bounding box.
    ///
    /// If a feature has multiple spatial geometry properties, it is the
    /// decision of the server whether only a single spatial geometry property
    /// is used to determine the extent or all relevant geometries.
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[param(value_type = Bbox, style = Form, explode = false, nullable = false)]
    pub bbox: Option<Bbox>,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    #[param(value_type = String, nullable = false)]
    pub bbox_crs: Crs,
    /// Either a date-time or an interval. Date and time expressions adhere to
    /// RFC 3339. Intervals may be bounded or half-bounded (double-dots at start or end).
    ///
    /// Examples:
    ///
    /// * A date-time: "2018-02-12T23:20:50Z"
    /// * A bounded interval: "2018-02-12T00:00:00Z/2018-03-18T12:31:12Z"
    /// * Half-bounded intervals: "2018-02-12T00:00:00Z/.." or "../2018-03-18T12:31:12Z"
    ///
    /// Only features that have a temporal property that intersects the value
    /// of `datetime` are selected.
    ///
    /// If a feature has multiple temporal properties, it is the decision of
    /// the server whether only a single temporal property is used to determine
    /// the extent or all relevant temporal properties.
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[param(value_type = String)]
    pub datetime: Option<Datetime>,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    #[param(value_type = String)]
    pub crs: Crs,
    #[param(nullable = false)]
    pub filter: Option<String>,
    #[serde(default)]
    #[param(inline, nullable = false)]
    pub filter_lang: Option<FilterLang>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[param(value_type = String)]
    pub filter_crs: Option<Crs>,
    /// Parameters for filtering on feature properties
    #[serde(default, flatten)]
    pub additional_parameters: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum FilterLang {
    #[default]
    CqlText,
    CqlJson,
}
