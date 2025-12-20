use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{Collection, Crs, Link};

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Collections {
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_stamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_matched: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_returned: Option<u64>,
    pub collections: Vec<Collection>,
    #[serde(default)]
    #[schema(value_type = Vec<String>)]
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
