pub mod collection;
mod crs;
mod datetime;
mod exception;
mod link;

pub use self::crs::CRS;
pub use self::datetime::Datetime;
pub use self::exception::exception;
pub use self::link::*;

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
#[derive(Serialize, Deserialize, Clone)]
pub struct Conformance {
    pub conforms_to: Vec<String>,
}
