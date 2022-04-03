pub use tileset::*;
pub use tms::*;

mod tileset;
mod tms;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::common::Crs;

/// A 2DPoint in the CRS indicated elsewere
type Point2D = [f64; 2];

/// Ordered list of names of the dimensions defined in the CRS
type OrderedAxes = [String; 2];

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TitleDescriptionKeywords {
    /// Title of this resource entity, normally used for display to a human
    pub title: Option<String>,
    /// Brief narrative description of this resoource entity, normally available
    /// for display to a human
    pub description: Option<String>,
    /// Unordered list of one or more commonly used or formalized word(s) or
    /// phrase(s) used to describe this resource entity
    pub keywords: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct TileQuery {
    pub collections: Option<String>,
}

/// Minimum bounding rectangle surrounding a 2D resource in the CRS indicated elsewere
#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BoundingBox2D {
    pub lower_left: Point2D,
    pub upper_right: Point2D,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub crs: Option<Crs>,
    pub orderd_axes: Option<OrderedAxes>,
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::BoundingBox2D;

    #[test]
    fn deserialize_bounding_box() {
        let value = json!({"lowerLeft": [1,2], "upperRight": [3, 4]});
        let bbox: BoundingBox2D = serde_json::from_value(value).unwrap();
        dbg!(bbox);
    }
}
