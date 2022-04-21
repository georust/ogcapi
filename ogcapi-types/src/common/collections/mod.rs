mod collection;
mod extent;
mod query;

pub use collection::*;
pub use extent::*;
pub use query::Query;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};

use crate::common::{Crs, Links};

pub static CRS_REF: &str = "#/crs";

#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Collections {
    #[serde(default)]
    pub links: Links,
    pub time_stamp: Option<String>,
    pub number_matched: Option<u64>,
    pub number_returned: Option<u64>,
    pub collections: Vec<Collection>,
    #[serde(default)]
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub crs: Vec<Crs>,
}
