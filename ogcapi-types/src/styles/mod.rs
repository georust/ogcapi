mod mapbox;
mod symcore;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::Link;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Styles {
    pub styles: Vec<Style>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Style {
    pub id: String,
    pub title: Option<String>,
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stylesheet {
    pub id: String,
    pub value: Value,
}
