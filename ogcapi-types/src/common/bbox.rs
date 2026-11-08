use std::{fmt, str};

use serde::{Deserialize, Serialize};

use super::Crs;

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
/// (<http://www.opengis.net/def/crs/OGC/1.3/CRS84>) unless a different coordinate
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum Bbox {
    #[cfg_attr(feature = "utoipa", schema(value_type = [f64; 4]))]
    Bbox2D([f64; 4]),
    #[cfg_attr(feature = "utoipa", schema(value_type = [f64; 6]))]
    Bbox3D([f64; 6]),
}

impl Bbox {
    /// This box, its coordinates in `crs`, as a box in CRS84; `None` where
    /// `crs` is neither CRS84 (nor `CRS84h`) nor Web Mercator, or the box lies
    /// outside its extent or has its south north of its north. A box of six
    /// values keeps its heights.
    #[must_use]
    pub fn to_crs84(&self, crs: &Crs) -> Option<Bbox> {
        let [west, south, east, north] = self.horizontal();
        let projection = crs.projection()?;
        let [min_x, min_y, max_x, max_y] = projection.extent();
        let within = [west, east].iter().all(|x| (min_x..=max_x).contains(x))
            && [south, north].iter().all(|y| (min_y..=max_y).contains(y));
        if !within || south > north {
            return None;
        }
        let (west, south) = projection.inverse(west, south);
        let (east, north) = projection.inverse(east, north);
        Some(match *self {
            Bbox::Bbox2D(_) => Bbox::Bbox2D([west, south, east, north]),
            Bbox::Bbox3D([_, _, bottom, _, _, top]) => {
                Bbox::Bbox3D([west, south, bottom, east, north, top])
            }
        })
    }

    /// The box of `positions`, in three dimensions where every position has a
    /// height; `None` for no positions. A position of fewer than two values
    /// is left out.
    #[cfg(feature = "features")]
    #[must_use]
    pub(crate) fn extent<'a>(positions: impl IntoIterator<Item = &'a [f64]>) -> Option<Bbox> {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let mut any = false;
        let mut heights = true;
        for position in positions {
            let [x, y, rest @ ..] = position else {
                continue;
            };
            any = true;
            heights &= !rest.is_empty();
            for (axis, value) in [x, y].into_iter().chain(rest.first()).enumerate() {
                min[axis] = min[axis].min(*value);
                max[axis] = max[axis].max(*value);
            }
        }
        any.then(|| {
            if heights {
                Bbox::Bbox3D([min[0], min[1], min[2], max[0], max[1], max[2]])
            } else {
                Bbox::Bbox2D([min[0], min[1], max[0], max[1]])
            }
        })
    }

    /// The box holding this one and `other`, in three dimensions where both
    /// are; every longitude where one of them crosses the antimeridian (its
    /// west east of its east).
    #[must_use]
    pub fn union(&self, other: &Bbox) -> Bbox {
        let [aw, a_s, ae, an] = self.horizontal();
        let [bw, b_s, be, bn] = other.horizontal();
        let (west, east) = if aw > ae || bw > be {
            (-180.0, 180.0)
        } else {
            (aw.min(bw), ae.max(be))
        };
        let (south, north) = (a_s.min(b_s), an.max(bn));
        if let (Bbox::Bbox3D(a), Bbox::Bbox3D(b)) = (self, other) {
            Bbox::Bbox3D([west, south, a[2].min(b[2]), east, north, a[5].max(b[5])])
        } else {
            Bbox::Bbox2D([west, south, east, north])
        }
    }

    /// Whether this box and `other` meet, on the axes both have. A box in
    /// CRS84 whose west lies east of its east crosses the antimeridian.
    #[must_use]
    pub fn intersects(&self, other: &Bbox) -> bool {
        let [aw, a_s, ae, an] = self.horizontal();
        let [bw, b_s, be, bn] = other.horizontal();
        let heights = if let (Bbox::Bbox3D(a), Bbox::Bbox3D(b)) = (self, other) {
            a[2] <= b[5] && b[2] <= a[5]
        } else {
            true
        };
        let longitudes = |west: f64, east: f64| {
            if west <= east {
                [(west, east), (west, east)]
            } else {
                [(west, 180.0), (-180.0, east)]
            }
        };
        let longitudes_meet = longitudes(aw, ae)
            .iter()
            .any(|a| longitudes(bw, be).iter().any(|b| a.0 <= b.1 && b.0 <= a.1));
        heights && a_s <= bn && b_s <= an && longitudes_meet
    }

    /// The box on its first two axes, `[west, south, east, north]`.
    fn horizontal(&self) -> [f64; 4] {
        match *self {
            Bbox::Bbox2D(bbox) => bbox,
            Bbox::Bbox3D([west, south, _, east, north, _]) => [west, south, east, north],
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

        // check lower <= upper on axis 3
        if n == 6 && numbers[2] > numbers[5] {
            return Err("The lower value of coordinate axis 3 is above its upper value");
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
        match value.len() {
            4 => Ok(Bbox::Bbox2D([value[0], value[1], value[2], value[3]])),
            6 => Ok(Bbox::Bbox3D([
                value[0], value[1], value[2], value[3], value[4], value[5],
            ])),
            _ => Err("Bbox can only be of lenth 4 or 6!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[cfg(feature = "features")]
    #[test]
    fn the_extent_of_positions() {
        let flat: [&[f64]; 3] = [&[1.0, 2.0], &[-1.0, 5.0, 7.0], &[0.0]];
        assert_eq!(
            Bbox::extent(flat),
            Some(Bbox::Bbox2D([-1.0, 2.0, 1.0, 5.0]))
        );
        let high: [&[f64]; 2] = [&[1.0, 2.0, 3.0, 9.0], &[-1.0, 5.0, -3.0]];
        assert_eq!(
            Bbox::extent(high),
            Some(Bbox::Bbox3D([-1.0, 2.0, -3.0, 1.0, 5.0, 3.0]))
        );
        assert_eq!(Bbox::extent(std::iter::empty()), None);
    }

    #[test]
    fn a_union_and_an_intersection() {
        let a = Bbox::Bbox3D([0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        let b = Bbox::Bbox3D([2.0, 2.0, 2.0, 3.0, 3.0, 3.0]);
        assert_eq!(a.union(&b), Bbox::Bbox3D([0.0, 0.0, 0.0, 3.0, 3.0, 3.0]));
        let flat = Bbox::Bbox2D([0.5, 0.5, 2.5, 2.5]);
        assert_eq!(a.union(&flat), Bbox::Bbox2D([0.0, 0.0, 2.5, 2.5]));
        // across the antimeridian, every longitude
        let pacific = Bbox::Bbox2D([170.0, -10.0, -170.0, 10.0]);
        assert_eq!(
            pacific.union(&Bbox::Bbox2D([0.0, 0.0, 10.0, 1.0])),
            Bbox::Bbox2D([-180.0, -10.0, 180.0, 10.0])
        );

        assert!(!a.intersects(&b));
        assert!(a.intersects(&flat) && flat.intersects(&b));
        // apart in height only
        assert!(!a.intersects(&Bbox::Bbox3D([0.0, 0.0, 2.0, 1.0, 1.0, 3.0])));
        // across the antimeridian
        assert!(pacific.intersects(&Bbox::Bbox2D([175.0, 0.0, 176.0, 1.0])));
        assert!(pacific.intersects(&Bbox::Bbox2D([-179.0, 0.0, -178.0, 1.0])));
        assert!(!pacific.intersects(&Bbox::Bbox2D([0.0, 0.0, 1.0, 1.0])));
        assert!(pacific.intersects(&Bbox::Bbox2D([160.0, 0.0, -175.0, 1.0])));
    }

    #[test]
    fn a_box_in_crs84() {
        let crs84 = Crs::default2d();
        let switzerland = Bbox::Bbox2D([5.96, 45.82, 10.49, 47.81]);
        assert_eq!(switzerland.to_crs84(&crs84), Some(switzerland.clone()));
        // Across the antimeridian, but not across a pole.
        let pacific = Bbox::Bbox2D([170.0, -10.0, -170.0, 10.0]);
        assert!(pacific.to_crs84(&crs84).is_some());
        assert!(
            Bbox::Bbox2D([0.0, 10.0, 1.0, -10.0])
                .to_crs84(&crs84)
                .is_none()
        );
        assert!(
            Bbox::Bbox2D([0.0, 0.0, 190.0, 1.0])
                .to_crs84(&crs84)
                .is_none()
        );
        // Web Mercator within its square, and no other system.
        let mercator = Crs::from_epsg(3857);
        let [west, south, east, north] =
            match Bbox::Bbox2D([0.0, 0.0, 1_113_194.9, 1_118_890.0]).to_crs84(&mercator) {
                Some(Bbox::Bbox2D(bbox)) => bbox,
                other => panic!("{other:?}"),
            };
        assert!(west.abs() < 1e-9 && south.abs() < 1e-9);
        assert!((east - 10.0).abs() < 1e-6 && (north - 10.0).abs() < 1e-4);
        // heights are kept
        assert!(matches!(
            Bbox::Bbox3D([0.0, 0.0, -5.0, 1.0, 1.0, 5.0]).to_crs84(&mercator),
            Some(Bbox::Bbox3D([_, _, -5.0, _, _, 5.0]))
        ));
        assert!(
            Bbox::Bbox2D([0.0, 0.0, 3e7, 1.0])
                .to_crs84(&mercator)
                .is_none()
        );
        assert!(switzerland.to_crs84(&Crs::from_epsg(2056)).is_none());
    }

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
