use std::{fmt, str};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ListParam(pub Vec<String>);

impl fmt::Display for ListParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}

impl str::FromStr for ListParam {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ListParam(
            s.split(',').map(|s| s.trim().to_owned()).collect(),
        ))
    }
}

impl From<&[&str]> for ListParam {
    fn from(l: &[&str]) -> Self {
        ListParam(l.iter().map(|s| s.trim().to_string()).collect())
    }
}
