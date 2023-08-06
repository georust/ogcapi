use std::{fmt, str};

use serde::{Deserialize, Serialize};

/// Default CRS for coordinates without height
pub const OGC_CRS84: &str = "http://www.opengis.net/def/crs/OGC/1.3/CRS84";

/// Default CRS for coordinates with height
// pub const OGC_CRS84H: &str = "http://www.opengis.net/def/crs/OGC/0/CRS84h";

/// Coordinate Reference System (CRS)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Crs {
    pub authority: Authority,
    pub version: String,
    pub code: String,
}

impl Crs {
    pub fn new(authority: Authority, version: impl ToString, code: impl ToString) -> Crs {
        Crs {
            authority,
            version: version.to_string(),
            code: code.to_string(),
        }
    }

    pub fn from_epsg(code: i32) -> Self {
        Crs::new(Authority::EPSG, "0", code)
    }

    pub fn from_srid(code: i32) -> Self {
        if code == 4326 {
            Crs::default()
        } else {
            Crs::new(Authority::EPSG, "0", code)
        }
    }

    pub fn to_urn(&self) -> String {
        format!(
            "urn:ogc:def:crs:{authority}:{version}:{code}",
            authority = self.authority,
            version = self.version,
            code = self.code
        )
    }

    pub fn to_epsg(&self) -> Option<Crs> {
        match self.authority {
            Authority::OGC => match self.code.as_str() {
                "CRS84h" => Some(Crs::new(Authority::EPSG, "0", "4979")),
                _ => None,
            },
            Authority::EPSG => Some(self.to_owned()),
        }
    }

    pub fn as_epsg(&self) -> Option<i32> {
        match self.authority {
            Authority::OGC => match self.code.as_str() {
                "CRS84h" => Some(4979),
                _ => panic!("Unable to extract epsg code from `{}`", self),
            },
            Authority::EPSG => self.code.parse().ok(),
        }
    }

    pub fn as_srid(&self) -> i32 {
        match self.authority {
            Authority::OGC => match self.code.as_str() {
                "CRS84" => 4326,
                "CRS84h" => 4979,
                _ => panic!("Unable to extract epsg code from `{}`", self),
            },
            Authority::EPSG => self.code.parse().unwrap(),
        }
    }

    /// "AUTHORITY:CODE", like "EPSG:25832"
    pub fn as_known_crs(&self) -> String {
        format!("{}:{}", self.authority, self.code)
    }
}

impl fmt::Display for Crs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "http://www.opengis.net/def/crs/{authority}/{version}/{code}",
            authority = self.authority,
            version = self.version,
            code = self.code
        )
    }
}

impl str::FromStr for Crs {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = if s.starts_with("urn") {
            s.trim_start_matches("urn:ogc:def:crs:")
                .split(':')
                .collect()
        } else {
            s.trim_start_matches("http://www.opengis.net/def/crs/")
                .split('/')
                .collect()
        };
        match parts.len() {
            3 => Ok(Crs::new(Authority::from_str(parts[0])?, parts[1], parts[2])),
            _ => Err(format!("Unable to parse CRS from `{s}`!")),
        }
    }
}

impl Default for Crs {
    fn default() -> Crs {
        Crs {
            authority: Authority::OGC,
            version: "1.3".to_string(),
            code: "CRS84".to_string(),
        }
    }
}

/// CRS Authorities
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
pub enum Authority {
    OGC,
    EPSG,
}

impl fmt::Display for Authority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Authority::OGC => write!(f, "OGC"),
            Authority::EPSG => write!(f, "EPSG"),
        }
    }
}

impl str::FromStr for Authority {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OGC" => Ok(Authority::OGC),
            "EPSG" => Ok(Authority::EPSG),
            _ => Err("Unknown crs authority!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::common::{Crs, OGC_CRS84};

    #[test]
    fn parse_crs() {
        let crs = Crs::from_str(OGC_CRS84).unwrap();
        assert_eq!(format!("{:#}", crs), OGC_CRS84)
    }

    #[test]
    fn from_epsg() {
        let code = 4979;
        let crs = Crs::from_epsg(code);
        assert_eq!(
            crs.to_string(),
            "http://www.opengis.net/def/crs/EPSG/0/4979".to_string()
        );

        let epsg = crs.as_epsg();
        assert_eq!(epsg, Some(code));
    }

    #[test]
    fn into_epsg() {
        let crs = Crs::from_epsg(4979);
        assert_eq!(
            crs.to_string(),
            "http://www.opengis.net/def/crs/EPSG/0/4979".to_string()
        )
    }

    #[test]
    fn to_epsg() {
        let crs = Crs::from_str("http://www.opengis.net/def/crs/EPSG/0/4979").unwrap();
        assert_eq!(crs.to_epsg(), Some(Crs::from_epsg(4979)))
    }
}
