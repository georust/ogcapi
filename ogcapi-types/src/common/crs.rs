use std::{convert::TryFrom, fmt, str};

use serde::{Deserialize, Serialize};

/// Default CRS for coordinates without height
pub const OGC_CRS84: &str = "http://www.opengis.net/def/crs/OGC/1.3/CRS84";

/// Default CRS for coordinates with height
pub const OGC_CRS84H: &str = "http://www.opengis.net/def/crs/OGC/0/CRS84h";

/// Coordinate Reference System (CRS)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
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

    pub fn ogc_to_epsg(&self) -> Option<Crs> {
        match self.authority {
            Authority::OGC => match self.code.as_str() {
                "CRS84" => Some(4326.into()),
                "CRS84h" => Some(4979.into()),
                _ => None,
            },
            Authority::EPSG => Some(self.to_owned()),
        }
    }

    pub fn to_ogc_urn(&self) -> String {
        format!(
            "urn:ogc:def:crs:{authority}:{version}:{code}",
            authority = self.authority,
            version = self.version,
            code = self.code
        )
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
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: handle urn
        let parts: Vec<&str> = s
            .trim_start_matches("http://www.opengis.net/def/crs/")
            .split('/')
            .collect();
        match parts.len() {
            3 => Ok(Crs::new(Authority::from_str(parts[0])?, parts[1], parts[2])),
            _ => Err("Unable to parse CRS from string!"),
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

impl From<i32> for Crs {
    fn from(epsg_code: i32) -> Self {
        Crs::new(Authority::EPSG, "0", &epsg_code.to_string())
    }
}

impl TryFrom<Crs> for i32 {
    type Error = &'static str;

    fn try_from(crs: Crs) -> Result<i32, &'static str> {
        match crs.authority {
            Authority::OGC => match crs.code.as_str() {
                "CRS84" => Ok(4326),
                "CRS84h" => Ok(4979),
                _ => Err("Unable to extract epsg code"),
            },
            Authority::EPSG => Ok(crs.code.parse().unwrap()),
        }
    }
}

/// CRS Authorities
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Hash, Eq)]
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
    use std::{convert::TryInto, str::FromStr};

    use crate::common::{Crs, OGC_CRS84};

    #[test]
    fn parse_crs() {
        let crs = Crs::from_str(OGC_CRS84).unwrap();
        assert_eq!(format!("{:#}", crs), OGC_CRS84)
    }

    #[test]
    fn from_epsg() {
        let code = 4979;
        let crs: Crs = code.into();
        assert_eq!(
            crs.to_string(),
            "http://www.opengis.net/def/crs/EPSG/0/4979".to_string()
        );

        let epsg: i32 = crs.try_into().unwrap();
        assert_eq!(epsg, code);

        let epsg: i32 = Crs::default().try_into().unwrap();
        assert_eq!(epsg, 4326)
    }

    #[test]
    fn into_epsg() {
        let crs: Crs = 4979.into();
        assert_eq!(
            crs.to_string(),
            "http://www.opengis.net/def/crs/EPSG/0/4979".to_string()
        )
    }

    #[test]
    fn ogc_to_epsg() {
        let crs = Crs::from_str("http://www.opengis.net/def/crs/EPSG/0/4979").unwrap();
        assert_eq!(crs.ogc_to_epsg(), Some(4979.into()))
    }
}
