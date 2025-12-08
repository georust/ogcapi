use super::{
    TransmissionMode,
    description::{InputDescription, OutputDescription},
};
use crate::common::Link;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Process summary
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSummary {
    pub id: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub job_control_options: Vec<JobControlOptions>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_transmission: Vec<TransmissionMode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum JobControlOptions {
    SyncExecute,
    AsyncExecute,
    Dismiss,
}

/// Information about the available processes
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ProcessList {
    pub processes: Vec<ProcessSummary>,
    pub links: Vec<Link>,
}

/// Full process description
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct Process {
    #[serde(flatten)]
    pub summary: ProcessSummary,
    #[schema(required = false)]
    pub inputs: HashMap<String, InputDescription>,
    #[schema(required = false)]
    pub outputs: HashMap<String, OutputDescription>,
}

/// Defines the authentication requirement for a Process
///
/// The OpenAPI specification currently supports the following security schemes:
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ProcessAuthRequirement {
    #[default]
    NoAuth,
    /// HTTP authentication,
    Http,
    /// an API key (either as a header or as a query parameter),
    ApiKey {
        name: String,
        r#in: ProcessAuthLocation,
    },
    /// OAuth2’s common flows (implicit, password, application and access code) as defined in RFC6749, and
    OAuth2, // TODO: add fields
    /// OpenID Connect Discovery.
    OpenIDConnect, // TODO: add fields
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessAuthLocation {
    Header,
    Query,
    Cookie,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessAuth {
    None,
    Http(ProcessAuthHttp),
    ApiKey(String),
    // TODO: add others
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessAuthHttp {
    pub r#type: ProcessAuthHttpType,
    pub credentials: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum ProcessAuthHttpType {
    #[serde(alias = "basic")] // lowercase to match standard HTTP auth scheme names
    Basic,
    #[serde(alias = "bearer")] // lowercase to match standard HTTP auth scheme names
    Bearer,
    // TODO: add others
}
