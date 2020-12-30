mod bbox;
mod crs;
mod datetime;
mod exception;
mod link;

use std::str::FromStr;

pub use self::bbox::BBOX;
pub use self::crs::CRS;
pub use self::datetime::Datetime;
pub use self::exception::exception;
pub use self::link::*;

use serde::{Deserialize, Serialize};
use tide::http::Mime;

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

// pub(crate) mod string {
//     use std::fmt::Display;
//     use std::str::FromStr;

//     use serde::{de, Deserialize, Deserializer, Serializer};

//     pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         T: Display,
//         S: Serializer,
//     {
//         serializer.collect_str(value)
//     }

//     pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
//     where
//         T: FromStr,
//         T::Err: Display,
//         D: Deserializer<'de>,
//     {
//         String::deserialize(deserializer)?
//             .parse()
//             .map_err(de::Error::custom)
//     }
// }

/// Content Type
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ContentType {
    #[serde(rename = "application/json")]
    JSON,
    #[serde(rename = "application/geo+json")]
    GEOJSON,
    #[serde(rename = "application/vnd.oai.openapi+json;version=3.0")]
    OPENAPI,
}

impl Into<Mime> for ContentType {
    fn into(self) -> Mime {
        let content_type = serde_json::to_string(&self).expect("Serialized content type");
        Mime::from_str(&content_type.as_str()).expect("Parse into media type")
    }
}

#[test]
fn parse_into_mime() {
    let _mime_json: Mime = ContentType::JSON.into();
    let _mime_geojson: Mime = ContentType::GEOJSON.into();
    let _mime_openapi: Mime = ContentType::OPENAPI.into();
}
