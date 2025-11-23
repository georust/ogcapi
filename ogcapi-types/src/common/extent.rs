use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize, ser::SerializeSeq, ser::Serializer};
use serde_with::DisplayFromStr;
use utoipa::ToSchema;

use crate::common::{Bbox, Crs};

/// The extent of the features in the collection. In the Core only spatial and
/// temporal extents are specified. Extensions may add additional members to
/// represent other extents, for example, thermal or pressure ranges.
#[derive(Serialize, Deserialize, ToSchema, Default, Debug, PartialEq, Clone)]
pub struct Extent {
    pub spatial: SpatialExtent,
    pub temporal: TemporalExtent,
}

/// The spatial extent of the features in the collection.
#[serde_with::serde_as]
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct SpatialExtent {
    /// One or more bounding boxes that describe the spatial extent of the
    /// dataset. In the Core only a single bounding box is supported. Extensions
    /// may support additional areas. If multiple areas are provided, the union
    /// of the bounding boxes describes the spatial extent.
    #[serde(default)]
    pub bbox: Vec<Bbox>,
    /// Coordinate reference system of the coordinates in the spatial extent
    /// (property `bbox`). The default reference system is WGS 84 longitude/latitude.
    /// In the Core this is the only supported coordinate reference system.
    /// Extensions may support additional coordinate reference systems and add
    /// additional enum values.
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = String)]
    pub crs: Option<Crs>,
}

impl Default for SpatialExtent {
    fn default() -> Self {
        Self {
            bbox: vec![Bbox::Bbox2D([-180.0, -90.0, 180.0, 90.0])],
            crs: Default::default(),
        }
    }
}

/// The temporal extent of the features in the collection.
#[serde_with::serde_as]
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
pub struct TemporalExtent {
    /// One or more time intervals that describe the temporal extent of the
    /// dataset. The value `null` is supported and indicates an unbounded
    /// interval end. In the Core only a single time interval is supported.
    /// Extensions may support multiple intervals. If multiple intervals are
    /// provided, the union of the intervals describes the temporal extent.
    #[serde(serialize_with = "serialize_interval")]
    pub interval: Vec<[Option<DateTime<Utc>>; 2]>,
    /// Coordinate reference system of the coordinates in the temporal extent
    /// (property `interval`). The default reference system is the Gregorian
    /// calendar. In the Core this is the only supported temporal coordinate
    /// reference system. Extensions may support additional temporal coordinate
    /// reference systems and add additional enum values.
    #[serde(default = "default_trs")]
    pub trs: String,
}

impl Default for TemporalExtent {
    fn default() -> Self {
        Self {
            interval: vec![[None, None]],
            trs: default_trs(),
        }
    }
}

pub(crate) fn serialize_interval<S>(
    interval: &Vec<[Option<DateTime<Utc>>; 2]>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut outer_seq = serializer.serialize_seq(Some(interval.len()))?;
    for inner_vec in interval {
        let serialized_inner_vec: Vec<_> = inner_vec
            .iter()
            .map(|item| {
                item.as_ref()
                    .map(|dt| dt.to_rfc3339_opts(SecondsFormat::Secs, true))
            })
            .collect();

        outer_seq.serialize_element(&serialized_inner_vec)?;
    }
    outer_seq.end()
}

fn default_trs() -> String {
    "http://www.opengis.net/def/uom/ISO-8601/0/Gregorian".to_string()
}
