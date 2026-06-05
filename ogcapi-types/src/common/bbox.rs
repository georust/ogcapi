use std::{fmt, ops::RangeInclusive, str};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Each bounding box is provided as four or six numbers, depending on
/// whether the coordinate reference system includes a vertical axis
/// (height or depth):
///
/// * Lower left corner, coordinate axis 1
///
/// * Lower left corner, coordinate axis 2
///
/// * Minimum value, coordinate axis 3 (optional)
///
/// * Upper right corner, coordinate axis 1
///
/// * Upper right corner, coordinate axis 2
///
/// * Maximum value, coordinate axis 3 (optional)
///
/// The coordinate reference system of the values is WGS 84 longitude/latitude
/// (http://www.opengis.net/def/crs/OGC/1.3/CRS84) unless a different coordinate
/// reference system is specified in `crs`.
///
/// For WGS 84 longitude/latitude the values are in most cases the sequence of
/// minimum longitude, minimum latitude, maximum longitude and maximum latitude.
/// However, in cases where the box spans the antimeridian the first value
/// (west-most box edge) is larger than the third value (east-most box edge).
///
/// If the vertical axis is included, the third and the sixth number are
/// the bottom and the top of the 3-dimensional bounding box.
///
/// If a feature has multiple spatial geometry properties, it is the decision of the
/// server whether only a single spatial geometry property is used to determine
/// the extent or all relevant geometries.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone, Copy)]
#[serde(untagged)]
pub enum Bbox {
    #[schema(value_type = [f64; 4])]
    Bbox2D([f64; 4]),
    #[schema(value_type = [f64; 6])]
    Bbox3D([f64; 6]),
}

impl Bbox {
    /// Create an empty 2D bounding box.
    pub fn new_empty_2d() -> Self {
        Self::Bbox2D([
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ])
    }

    /// Create an empty 3D bounding box.
    pub fn new_empty_3d() -> Self {
        Self::Bbox3D([
            f64::INFINITY,
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ])
    }

    /// Get the bounding box numbers as slice.
    pub fn as_slice(&self) -> &[f64] {
        match self {
            Bbox::Bbox2D(bbox) => bbox.as_slice(),
            Bbox::Bbox3D(bbox) => bbox.as_slice(),
        }
    }

    /// Get the bounding box numbers as mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [f64] {
        match self {
            Bbox::Bbox2D(bbox) => bbox.as_mut_slice(),
            Bbox::Bbox3D(bbox) => bbox.as_mut_slice(),
        }
    }

    /// Get the inclusive interval for an `axis`: 1 = x/easting, 2 = y/northing, 3 = z/height.
    pub fn interval(&self, axis: usize) -> RangeInclusive<f64> {
        let numbers = self.as_slice();
        let offset = numbers.len() / 2;
        match axis {
            1 => numbers[0]..=numbers[offset],
            2 => numbers[1]..=numbers[offset + 1],
            3 => {
                if offset == 3 {
                    numbers[1]..=numbers[offset + 1]
                } else {
                    // full / unbound
                    f64::NEG_INFINITY..=f64::INFINITY
                }
            }
            x => panic!("axis must be 1, 2 or 3, got {x}"),
        }
    }

    /// Test if the bounding box intersects with `other`.
    ///
    /// Gracefully handles 2D and 3D mixups by ignoring axis 3 if not present in both.
    pub fn intersects(&self, other: &Bbox) -> bool {
        match (self, other) {
            (
                Bbox::Bbox2D([axmin, aymin, axmax, aymax]),
                Bbox::Bbox2D([bxmin, bymin, bxmax, bymax]),
            )
            | (
                Bbox::Bbox2D([axmin, aymin, axmax, aymax]),
                Bbox::Bbox3D([bxmin, bymin, _, bxmax, bymax, _]),
            )
            | (
                Bbox::Bbox3D([axmin, aymin, _, axmax, aymax, _]),
                Bbox::Bbox2D([bxmin, bymin, bxmax, bymax]),
            ) => !(axmin > bxmax || axmax < bxmin || aymin > bymax || aymax < bymin),
            (
                Bbox::Bbox3D([axmin, aymin, azmin, axmax, aymax, azmax]),
                Bbox::Bbox3D([bxmin, bymin, bzmin, bxmax, bymax, bzmax]),
            ) => {
                !(axmin > bxmax
                    || axmax < bxmin
                    || aymin > bymax
                    || aymax < bymin
                    || azmin > bzmax
                    || azmax < bzmin)
            }
        }
    }

    /// Test if the bounding box contains `other`.
    ///
    /// Gracefully handles 2D and 3D mixups by ignoring axis 3 if not present in both.
    pub fn contains(&self, other: &Bbox) -> bool {
        match (self, other) {
            (
                Bbox::Bbox2D([axmin, aymin, axmax, aymax]),
                Bbox::Bbox2D([bxmin, bymin, bxmax, bymax]),
            )
            | (
                Bbox::Bbox2D([axmin, aymin, axmax, aymax]),
                Bbox::Bbox3D([bxmin, bymin, _, bxmax, bymax, _]),
            )
            | (
                Bbox::Bbox3D([axmin, aymin, _, axmax, aymax, _]),
                Bbox::Bbox2D([bxmin, bymin, bxmax, bymax]),
            ) => !(axmin > bxmin || axmax < bxmax || aymin > bymin || aymax < bymax),
            (
                Bbox::Bbox3D([axmin, aymin, azmin, axmax, aymax, azmax]),
                Bbox::Bbox3D([bxmin, bymin, bzmin, bxmax, bymax, bzmax]),
            ) => {
                !(axmin > bxmin
                    || axmax < bxmax
                    || aymin > bymin
                    || aymax < bymax
                    || azmin > bzmin
                    || azmax < bzmax)
            }
        }
    }

    /// Test if [Bbox] contains point.
    ///
    /// Gracefully handles 2D and 3D mixups by ignoring axis 3 if not present in both,
    /// but panics if point does have other dimensionality.
    pub fn contains_point(&self, point: &[f64]) -> bool {
        match (self, point) {
            (Bbox::Bbox2D([xmin, ymin, xmax, ymax]), [x, y])
            | (Bbox::Bbox3D([xmin, ymin, _, xmax, ymax, _]), [x, y]) => {
                !(xmin..=xmax).contains(&x) || !(ymin..=ymax).contains(&y)
            }
            (Bbox::Bbox3D([xmin, ymin, zmin, xmax, ymax, zmax]), [x, y, z]) => {
                !(xmin..=xmax).contains(&x)
                    || !(ymin..=ymax).contains(&y)
                    || !(zmin..=zmax).contains(&z)
            }
            _ => panic!("malformed point"),
        }
    }

    /// Extend the bounding box to contain `other`.
    ///
    /// Gracefully handles 2D and 3D mixups by ignoring axis 3 if not present in both.
    pub fn extend(&mut self, other: &Bbox) {
        match (self, other) {
            (
                Bbox::Bbox2D([axmin, aymin, axmax, aymax]),
                Bbox::Bbox2D([bxmin, bymin, bxmax, bymax]),
            )
            | (
                Bbox::Bbox2D([axmin, aymin, axmax, aymax]),
                Bbox::Bbox3D([bxmin, bymin, _, bxmax, bymax, _]),
            )
            | (
                Bbox::Bbox3D([axmin, aymin, _, axmax, aymax, _]),
                Bbox::Bbox2D([bxmin, bymin, bxmax, bymax]),
            ) => {
                *axmin = axmin.min(*bxmin);
                *axmax = axmax.max(*bxmax);
                *aymin = aymin.min(*bymin);
                *aymax = aymax.max(*bymax);
            }
            (
                Bbox::Bbox3D([axmin, aymin, azmin, axmax, aymax, azmax]),
                Bbox::Bbox3D([bxmin, bymin, bzmin, bxmax, bymax, bzmax]),
            ) => {
                *axmin = axmin.min(*bxmin);
                *axmax = axmax.max(*bxmax);
                *aymin = aymin.min(*bymin);
                *aymax = aymax.max(*bymax);
                *azmin = azmin.min(*bzmin);
                *azmax = azmax.max(*bzmax);
            }
        }
    }

    /// Extend the bounding box to contain `point`.
    ///
    /// Gracefully handles 2D and 3D mixups by ignoring axis 3 if not present in both.
    pub fn extend_point(&mut self, point: &[f64]) {
        match (self, point) {
            (Bbox::Bbox2D([xmin, ymin, xmax, ymax]), [x, y])
            | (Bbox::Bbox3D([xmin, ymin, _, xmax, ymax, _]), [x, y]) => {
                *xmin = xmin.min(*x);
                *xmax = xmax.max(*x);
                *ymin = ymin.min(*y);
                *ymax = ymax.max(*y);
            }
            (Bbox::Bbox3D([xmin, ymin, zmin, xmax, ymax, zmax]), [x, y, z]) => {
                *xmin = xmin.min(*x);
                *xmax = xmax.max(*x);
                *ymin = ymin.min(*y);
                *ymax = ymax.max(*y);
                *zmin = zmin.min(*z);
                *zmax = zmax.max(*z);
            }
            _ => panic!("malformed point"),
        }
    }

    /// Convert to 2D bounding box.
    pub fn as_2d(&self) -> Self {
        match self {
            Bbox::Bbox2D(_) => *self,
            Bbox::Bbox3D([xmin, ymin, _, xmax, ymax, _]) => {
                Self::Bbox2D([*xmin, *ymin, *xmax, *ymax])
            }
        }
    }
}

impl fmt::Display for Bbox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Bbox::Bbox2D(bbox) => write!(f, "{},{},{},{}", bbox[0], bbox[1], bbox[2], bbox[3]),
            Bbox::Bbox3D(bbox) => {
                write!(
                    f,
                    "{},{},{},{},{},{}",
                    bbox[0], bbox[1], bbox[2], bbox[3], bbox[4], bbox[5]
                )
            }
        }
    }
}

impl From<[f64; 4]> for Bbox {
    fn from(slice: [f64; 4]) -> Self {
        Bbox::Bbox2D(slice)
    }
}

impl From<[f64; 6]> for Bbox {
    fn from(slice: [f64; 6]) -> Self {
        Bbox::Bbox3D(slice)
    }
}

impl str::FromStr for Bbox {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numbers: Vec<f64> = s
            .split(',')
            .map(|d| d.trim().trim_matches(['[', ']']).parse::<f64>())
            .collect::<Result<Vec<f64>, std::num::ParseFloatError>>()
            .map_err(|_| "Unable to convert bbox coordinates to float")?;

        let n = numbers.len();

        // check number of coordinates
        if !(n == 4 || n == 6) {
            return Err("Expected 4 or 6 numbers");
        }

        // // check lower <= upper on axis 2
        // if (n == 4 && numbers[1] > numbers[3]) || (n == 6 && numbers[1] > numbers[4]) {
        //     // TODO: ensure this assumption is correct
        //     return Err("Lower value of coordinate axis 2 must be larger than upper value!");
        // }

        // check lower <= upper on axis 3
        if n == 6 && numbers[2] > numbers[5] {
            return Err("Lower value of coordinate axis 3 must be larger than upper value!");
        }

        match numbers.len() {
            4 => Ok(Bbox::Bbox2D([
                numbers[0], numbers[1], numbers[2], numbers[3],
            ])),
            6 => Ok(Bbox::Bbox3D([
                numbers[0], numbers[1], numbers[2], numbers[3], numbers[4], numbers[5],
            ])),
            _ => Err("Expected 4 or 6 numbers"),
        }
    }
}

impl TryFrom<&[f64]> for Bbox {
    type Error = &'static str;

    fn try_from(value: &[f64]) -> Result<Self, Self::Error> {
        match *value {
            [x, y] => Ok(Bbox::Bbox2D([x, y, x, y])),
            [x, y, z] => Ok(Bbox::Bbox3D([x, y, z, x, y, z])),
            [xmin, ymin, xmax, ymax] => Ok(Bbox::Bbox2D([xmin, ymin, xmax, ymax])),
            [xmin, ymin, zmin, xmax, ymax, zmax] => {
                Ok(Bbox::Bbox3D([xmin, ymin, zmin, xmax, ymax, zmax]))
            }
            _ => Err("Bbox can only be of lenth 4 or 6!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn from() {
        let numbers = [160.6, -55.95, -170.0, -25.89];
        let _bbox: Bbox = numbers.into();
    }

    #[test]
    fn try_from() {
        let numbers = &[160.6, -55.95, -170.0, -25.89];
        let _bbox: Bbox = numbers.as_slice().try_into().unwrap();
    }

    #[test]
    fn from_str() {
        let s = "160.6,-55.95, -170, -25.89";
        let _bbox: Bbox = Bbox::from_str(s).unwrap();
    }

    #[test]
    fn serde_json() {
        let s = "[ 160.6, -55.95, -170, -25.89 ]";
        let bbox: Bbox = serde_json::from_str(s).unwrap();

        match bbox {
            Bbox::Bbox2D { .. } => {}
            Bbox::Bbox3D { .. } => panic!("expected bbox to be 2 dimensional"),
        }
        assert_eq!(
            "[160.6,-55.95,-170.0,-25.89]",
            serde_json::to_string(&bbox).unwrap()
        );
    }
}
