use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::Links;

use super::execute::InlineOrRefData;

#[derive(Serialize, Deserialize, Debug, Default)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Results {
    #[serde(flatten)]
    results: HashMap<String, InlineOrRefData>,
}
