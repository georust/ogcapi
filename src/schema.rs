use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LandingPage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub links: Vec<Link>
}

#[derive(Serialize, Deserialize, Default)]
pub struct Link {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<i32>,
}


#[derive(Serialize, Deserialize)]
pub struct Conformance {
    #[serde(rename(deserialize = "conformsTo"))]
    pub conforms_to: Vec<String>,
}

// #[derive(Serialize, Deserialize)]
// pub struct ServerError {
//     pub code: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub description: Option<String>
// }