use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use utoipa::ToSchema;

use super::{Collection, Crs, Link};

#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Collections {
    #[serde(default)]
    pub links: Vec<Link>,
    pub time_stamp: Option<String>,
    pub number_matched: Option<u64>,
    pub number_returned: Option<u64>,
    pub collections: Vec<Collection>,
    #[serde(default)]
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = String)]
    pub crs: Vec<Crs>,
}

impl Collections {
    pub fn new(collections: Vec<Collection>) -> Self {
        let count = collections.len();
        Collections {
            links: Vec::new(),
            time_stamp: Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
            number_matched: None,
            number_returned: Some(count as u64),
            collections,
            crs: Vec::new(),
        }
    }
}
