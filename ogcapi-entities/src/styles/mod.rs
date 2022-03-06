mod mapbox;
mod symcore;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;

use crate::common::Links;

#[derive(Serialize, Deserialize, Debug)]
pub struct Styles {
    pub styles: Vec<Style>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Style {
    pub id: String,
    pub title: Option<String>,
    pub links: Json<Links>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Stylesheet {
    pub id: String,
    pub value: Json<Value>,
}
