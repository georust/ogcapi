use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// default CRS
pub static EPSG_WGS84: &str = "http://www.opengis.net/def/crs/EPSG/0/4326"; // for geojson, web and postgis
pub static EPSG_4979: &str = "http://www.opengis.net/def/crs/EPSG/0/4979"; // for coordinates with height
// static OGC_CRS84: &str = "http://www.opengis.net/def/crs/OGC/1.3/CRS84"; // for coordinates without height
// static OGC_CRS84h: &str = "http://www.opengis.net/def/crs/OGC/0/CRS84h"; // for coordinates with height

#[derive(Serialize, Deserialize, Default, Debug, Clone, sqlx::Type)]
pub struct CRS {
    authority: String,
    version: String,
    code: String,
}

impl CRS {
    fn new(authority: &str, version: &str, code: &str) -> CRS {
        CRS {
            authority: authority.to_string(),
            version: version.to_owned(),
            code: code.to_owned(),
        }
    }

    fn as_ogc_urn(&self) -> String {
        format!(
            "urn:ogc:def:crs:{authority}:{version}:{code}",
            authority = self.authority,
            version = self.version,
            code = self.code
        )
    }
}

impl fmt::Display for CRS {
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

impl FromStr for CRS {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s
            .trim_start_matches("http://www.opengis.net/def/crs/")
            .split("/")
            .collect();
        match parts.len() {
            3 => Ok(CRS::new(parts[0], parts[1], parts[2])),
            _ => Err("Unable to parse CRS from string!"),
        }
    }
}
