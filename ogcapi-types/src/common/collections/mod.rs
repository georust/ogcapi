mod collection;
mod query;

pub use collection::*;
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
    pub links: Links,
    pub time_stamp: Option<String>,
    pub number_matched: Option<usize>,
    pub number_returned: Option<usize>,
    pub collections: Vec<Collection>,
    #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
    pub crs: Option<Vec<Crs>>,
}
