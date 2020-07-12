pub mod crs;
pub mod link;

use crate::common::link::Link;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LandingPage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub links: Vec<Link>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize)]
pub struct Conformance {
    pub conforms_to: Vec<String>,
}

#[derive(Serialize)]
pub struct Exception {
    pub code: String,
    pub description: Option<String>,
}
