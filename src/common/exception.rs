use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Exception {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub status: Option<isize>,
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}
