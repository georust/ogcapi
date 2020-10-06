mod bbox;
mod crs;
mod datetime;
mod exception;
mod link;

pub use self::bbox::BBOX;
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

pub(crate) mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
