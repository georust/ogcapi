use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

use crate::common::Links;

use super::execute::InlineOrRefData;

#[derive(Serialize, Deserialize, Debug, Default, sqlx::FromRow)]
pub struct StatusInfo {
    #[serde(rename = "processID")]
    pub process_id: Option<String>,
    #[serde(rename = "jobID")]
    pub job_id: String,
    pub status: Json<StatusCode>,
    pub message: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub finished: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub progress: Option<i8>,
    pub links: Option<Json<Links>>,
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
