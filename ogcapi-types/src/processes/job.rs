use std::{collections::HashMap, ops::Deref};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{StringWithSeparator, formats::SpaceSeparator};

use crate::common::{Links, query::LimitOffsetPagination};

use super::execute::InlineOrRefData;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct JobList {
    jobs: Vec<StatusInfo>,
    links: Links,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StatusInfo {
    #[serde(rename = "processID", alias = "process_id")]
    pub process_id: Option<String>,
    #[serde(rename = "jobID", alias = "job_id")]
    pub job_id: String,
    pub status: StatusCode,
    pub message: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub finished: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub progress: Option<i8>,
    #[serde(default)]
    pub links: Links,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StatusCode {
    Accepted,
    Running,
    Successful,
    Failed,
    Dismissed,
}

impl Default for StatusCode {
    fn default() -> Self {
        Self::Accepted
    }
}

#[serde_with::serde_as]
#[derive(Deserialize, Debug)]
pub struct ResultsQuery {
    #[serde(flatten)]
    pub pagination: LimitOffsetPagination,
    #[serde(default)]
    #[serde_as(as = "Option<StringWithSeparator::<SpaceSeparator, String>>")]
    pub outputs: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Results {
    #[serde(flatten)]
    pub results: HashMap<String, InlineOrRefData>,
}

impl Deref for Results {
    type Target = HashMap<String, InlineOrRefData>;

    fn deref(&self) -> &Self::Target {
        &self.results
    }
}
