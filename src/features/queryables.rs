use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Queryables {
    queryables: Vec<Queryable>
}
#[derive(Serialize, Deserialize, Debug)]
struct Queryable {
    queryable: String,
    title: Option<String>,
    description: Option<String>,
    language: Option<String>, // default en
    r#type: String,
    #[serde(rename = "type-ref")]
    type_ref: String,
}