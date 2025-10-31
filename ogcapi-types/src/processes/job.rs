use std::{collections::HashMap, ops::Deref};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{StringWithSeparator, formats::SpaceSeparator};
use utoipa::ToSchema;

use crate::common::{Link, query::LimitOffsetPagination};

use super::execute::InlineOrRefData;

#[derive(Serialize, Deserialize, ToSchema, Debug, Default)]
pub struct JobList {
    pub jobs: Vec<StatusInfo>,
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Default)]
pub struct StatusInfo {
    #[schema(nullable = false)]
    #[serde(
        rename = "processID",
        alias = "process_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub process_id: Option<String>,
    #[schema(required = false)]
    #[serde(default)]
    pub r#type: JobType,
    #[serde(rename = "jobID", alias = "job_id")]
    pub job_id: String,
    pub status: StatusCode,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished: Option<DateTime<Utc>>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<DateTime<Utc>>,
    #[schema(nullable = false, value_type = isize, required = false, minimum = 0, maximum = 100)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[schema(default = "process")]
pub enum JobType {
    #[default]
    Process,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum StatusCode {
    #[default]
    Accepted,
    Running,
    Successful,
    Failed,
    Dismissed,
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

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
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
