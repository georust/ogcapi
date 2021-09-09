use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

use crate::common::core::Links;

use super::execute::InlineOrRefData;

#[derive(Serialize, Deserialize, Debug, Default, sqlx::FromRow)]
pub struct StatusInfo {
    #[serde(rename = "processID")]
    pub(crate) process_id: Option<String>,
    #[serde(rename = "jobID")]
    pub(crate) job_id: String,
    pub(crate) status: Json<StatusCode>,
    pub(crate) message: Option<String>,
    pub(crate) created: Option<DateTime<Utc>>,
    pub(crate) finished: Option<DateTime<Utc>>,
    pub(crate) updated: Option<DateTime<Utc>>,
    pub(crate) progress: Option<i8>,
    pub(crate) links: Option<Json<Links>>,
}

#[derive(Serialize, Deserialize, Debug)]
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
