use std::{
    f64::consts::PI,
    fmt,
    str::{self, FromStr},
};

use serde::{Deserialize, Serialize, de::Visitor};

/// Default CRS for coordinates without height
pub const OGC_CRS84: &str = "http://www.opengis.net/def/crs/OGC/1.3/CRS84";

/// Default CRS for coordinates with height
pub const OGC_CRS84H: &str = "http://www.opengis.net/def/crs/OGC/0/CRS84h";

/// Coordinate Reference System (CRS)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Crs {
    pub authority: Authority,
    pub version: String,
    pub code: String,
}

impl Crs {
    #[allow(
        clippy::needless_pass_by_value,
        reason = "This is a constructor, so passing by value is fine."
    )]
    pub fn new(authority: Authority, version: impl ToString, code: impl ToString) -> Self {
        Crs {
            authority,
            version: version.to_string(),
            code: code.to_string(),
        }
    }

    #[must_use]
    pub fn default2d() -> Self {
        Self::new(Authority::OGC, "1.3", "CRS84")
    }

    #[must_use]
    pub fn default3d() -> Self {
        Self::new(Authority::OGC, "0".to_string(), "CRS84h".to_string())
    }

    #[must_use]
    pub fn from_epsg(code: i32) -> Self {
        Crs::new(Authority::EPSG, "0", code)
    }

    #[must_use]
    pub fn from_srid(code: i32) -> Self {
        if code == 4326 {
            Crs::default2d()
        } else {
            Crs::new(Authority::EPSG, "0", code)
        }
    }

    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:ogc:def:crs:{authority}:{version}:{code}",
            authority = self.authority,
            version = self.version,
            code = self.code
        )
    }

    #[must_use]
    pub fn to_epsg(&self) -> Option<Crs> {
        match self.authority {
            Authority::OGC => match self.code.as_str() {
                "CRS84h" => Some(Crs::new(Authority::EPSG, "0", "4979")),
                _ => None,
            },
            Authority::EPSG => Some(self.to_owned()),
        }
    }

    #[must_use]
    pub fn as_epsg(&self) -> Option<i32> {
        match self.authority {
            Authority::OGC => match self.code.as_str() {
                "CRS84" => Some(4326),
                "CRS84h" => Some(4979),
                _ => {
                    log::error!("Unable to extract epsg code from `{self}`");
                    None
                }
            },
            Authority::EPSG => self.code.parse().ok(),
        }
    }

    #[must_use]
    pub fn as_srid(&self) -> Option<i32> {
        match self.authority {
            Authority::OGC => match self.code.as_str() {
                "CRS84" => Some(4326),
                "CRS84h" => Some(4979),
                _ => {
                    log::error!("Unable to extract srid code from `{self}`");
                    None
                }
            },
            Authority::EPSG => self.code.parse().ok(),
        }
    }

    /// "AUTHORITY:CODE", like "EPSG:25832"
    #[must_use]
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
        let parts: Vec<&str> = if s.starts_with("http") {
            s.trim_start_matches("http://www.opengis.net/def/crs/")
                .split('/')
                .collect()
        } else if s.starts_with("urn") {
            s.trim_start_matches("urn:ogc:def:crs:")
                .split(':')
                .collect()
        } else {
            s.split(':').collect()
        };
        match parts.len() {
            3 => Ok(Crs::new(
                Authority::from_str(parts[0])?,
                if parts[1].is_empty() { "0" } else { parts[1] },
                parts[2],
            )),
            2 => Ok(Crs::new(Authority::from_str(parts[0])?, "0", parts[1])),
            _ => Err(format!("Unable to parse CRS from `{s}`!")),
        }
    }
}

struct StrVisitior;

impl Visitor<'_> for StrVisitior {
    type Value = Crs;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a crs string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match Crs::from_str(v) {
            Ok(crs) => Ok(crs),
            Err(e) => Err(E::custom(e)),
        }
    }
}

impl<'de> Deserialize<'de> for Crs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(StrVisitior)
    }
}

impl Serialize for Crs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// CRS Authorities
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
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

impl FromStr for Authority {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OGC" => Ok(Authority::OGC),
            "EPSG" => Ok(Authority::EPSG),
            _ => Err("Unknown crs authority!"),
        }
    }
}

/// The radius of the sphere of Web Mercator (EPSG:3857), the semi-major axis
/// of WGS 84, in meters.
const EARTH_RADIUS: f64 = 6_378_137.0;

/// How CRS84 coordinates map to those of a coordinate reference system, for
/// CRS84 itself and Web Mercator (EPSG:3857). Web Mercator's are PROJ's
/// computation: its `webmerc` (the spherical formulas of merc.cpp), with its
/// longitudes wrapped (`adjlon`) and its degrees converted (`unitconvert`),
/// equal to the bit where the platform's math library rounds as PROJ's does
/// and a few ulps apart where it doesn't (nanometers).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    Crs84,
    WebMercator,
}

impl Projection {
    /// The coordinates of a CRS84 longitude and latitude. Within 1e-10 rad of
    /// a pole, which PROJ refuses, Web Mercator's northing is finite, far
    /// beyond its square.
    #[must_use]
    pub fn forward(self, lon: f64, lat: f64) -> (f64, f64) {
        match self {
            Projection::Crs84 => (lon, lat),
            Projection::WebMercator => (
                EARTH_RADIUS * adjlon(lon.to_radians()),
                EARTH_RADIUS * lat.to_radians().tan().asinh(),
            ),
        }
    }

    /// The CRS84 longitude and latitude of coordinates.
    #[must_use]
    pub fn inverse(self, x: f64, y: f64) -> (f64, f64) {
        match self {
            Projection::Crs84 => (x, y),
            // times the reciprocal of the radius, and divided by a degree's
            // radians rather than `to_degrees`, as PROJ rounds
            Projection::WebMercator => (
                adjlon(x * (1.0 / EARTH_RADIUS)) / (PI / 180.0),
                (y * (1.0 / EARTH_RADIUS)).sinh().atan() / (PI / 180.0),
            ),
        }
    }

    /// The extent of the coordinates, `[min_x, min_y, max_x, max_y]`.
    #[must_use]
    pub fn extent(self) -> [f64; 4] {
        match self {
            Projection::Crs84 => [-180.0, -90.0, 180.0, 90.0],
            Projection::WebMercator => {
                let edge = EARTH_RADIUS * PI;
                [-edge, -edge, edge, edge]
            }
        }
    }
}

/// A longitude in radians within [-π, π], as PROJ's `adjlon` wraps it, one
/// a little beyond (1e-12) kept as it is.
fn adjlon(lon: f64) -> f64 {
    if lon.abs() < PI + 1e-12 {
        return lon;
    }
    let lon = lon + PI;
    lon - 2.0 * PI * (lon / (2.0 * PI)).floor() - PI
}

impl Crs {
    /// The projection from CRS84 to this coordinate reference system, CRS84
    /// (`CRS84h` on its first two axes) or Web Mercator; `None` for another.
    #[must_use]
    pub fn projection(&self) -> Option<Projection> {
        match (&self.authority, self.code.as_str()) {
            (Authority::OGC, "CRS84" | "CRS84h") => Some(Projection::Crs84),
            (Authority::EPSG, "3857") => Some(Projection::WebMercator),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Projection;

    #[test]
    fn web_mercator_as_proj() {
        let web_mercator = Projection::WebMercator;
        for ((lon, lat), (x, y), (back_lon, back_lat)) in [
            (
                (7.43861, 46.95108),
                (828_062.277_409_752_7, 5_934_092.735_651_646),
                (7.43861, 46.95108),
            ),
            (
                (-180.0, 81.25),
                (-20_037_508.342_789_244, 16_394_199.262_074_44),
                (-180.0, 81.25),
            ),
            (
                (0.0, 89.999_999),
                (0.0, 118_358_352.966_090_25),
                (0.0, 89.999_999),
            ),
            (
                (179.999_999_999_9, -45.0),
                (20_037_508.342_778_113, -5_621_521.486_192_066),
                (179.999_999_999_9, -44.999_999_999_999_99),
            ),
        ] {
            let near = |(a, b): (f64, f64), (c, d): (f64, f64)| {
                let close = |u: f64, v: f64| (u - v).abs() <= 4.0 * f64::EPSILON * u.abs().max(1.0);
                close(a, c) && close(b, d)
            };
            assert!(near(web_mercator.forward(lon, lat), (x, y)), "{lon} {lat}");
            assert!(
                near(web_mercator.inverse(x, y), (back_lon, back_lat)),
                "{x} {y}"
            );
        }
        // longitudes wrapped, as PROJ wraps them
        let (x, _) = web_mercator.forward(190.0, 0.0);
        assert_eq!(x, web_mercator.forward(-170.0, 0.0).0);
    }

    #[test]
    fn a_projection_by_authority_and_code() {
        use std::str::FromStr;

        use crate::common::Crs;

        let crs84 = Crs::from_str("urn:ogc:def:crs:OGC::CRS84").unwrap();
        assert_eq!(crs84.projection(), Some(Projection::Crs84));
        assert_eq!(
            Crs::from_str("EPSG:3857").unwrap().projection(),
            Some(Projection::WebMercator)
        );
        assert_eq!(Crs::from_epsg(4326).projection(), None);
    }

    use std::str::FromStr;

    use crate::common::{Crs, OGC_CRS84};

    #[test]
    fn parse_crs() {
        let crs = Crs::from_str(OGC_CRS84).unwrap();
        assert_eq!(format!("{crs:#}"), OGC_CRS84);
    }

    #[test]
    fn serde_crs() {
        let original = format!("\"{OGC_CRS84}\"");
        let crs: Crs = serde_json::from_str(&original).unwrap();
        let crs_str = serde_json::to_string(&crs).unwrap();
        assert_eq!(crs_str, original);
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
        );
    }

    #[test]
    fn to_epsg() {
        let crs = Crs::from_str("http://www.opengis.net/def/crs/EPSG/0/4979").unwrap();
        assert_eq!(crs.to_epsg(), Some(Crs::from_epsg(4979)));
    }
}
