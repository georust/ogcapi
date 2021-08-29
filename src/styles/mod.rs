mod mapbox;
mod symcore;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;

use crate::common::Links;

#[derive(Serialize, Deserialize, Debug)]
pub struct Styles {
    pub(crate) styles: Vec<Style>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Style {
    id: String,
    title: Option<String>,
    links: Json<Links>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Stylesheet {
    id: String,
    pub value: Json<Value>,
}
