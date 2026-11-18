use std::{collections::HashMap, marker::PhantomData};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{StringWithSeparator, formats::SpaceSeparator};

use crate::common::{Link, query::LimitOffsetPagination};

use super::execute::InlineOrRefData;

#[derive(Serialize, Deserialize, Debug, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct JobList {
    pub jobs: Vec<StatusInfo>,
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct StatusInfo {
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(
        rename = "processID",
        alias = "process_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub process_id: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(required = false))]
    #[serde(default)]
    pub r#type: JobType,
    #[serde(rename = "jobID", alias = "job_id")]
    pub job_id: String,
    pub status: StatusCode,
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished: Option<DateTime<Utc>>,
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<DateTime<Utc>>,
    #[cfg_attr(feature = "utoipa", schema(nullable = false, value_type = isize, required = false, minimum = 0, maximum = 100))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

impl StatusInfo {
    /// The status of a job `job_id` accepted now, of no process yet.
    #[must_use]
    pub fn new(job_id: impl Into<String>) -> Self {
        Self {
            process_id: None,
            r#type: JobType::Process,
            job_id: job_id.into(),
            status: StatusCode::Accepted,
            message: None,
            created: Some(Utc::now()),
            finished: None,
            updated: None,
            progress: Some(0),
            links: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "utoipa", schema(default = "process"))]
pub enum JobType {
    #[default]
    Process,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(untagged)]
#[cfg_attr(feature = "utoipa", schema(title = "ExecuteResults"))]
pub enum Results {
    Json {
        #[serde(flatten)]
        results: HashMap<String, InlineOrRefData>,
    },
    #[cfg_attr(feature = "utoipa", schema(value_type = String, format = Binary))]
    // The `PhantomData` marker is required for proper OpenApi schema generation.
    Binary(PhantomData<Vec<u8>>),
}
