use std::{fmt, str};

use serde::{Deserialize, Serialize};

type Bbox2D = [f64; 4];
type Bbox3D = [f64; 6];

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Bbox {
    Bbox2D(Bbox2D),
    Bbox3D(Bbox3D),
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
            .map(|d| d.trim().parse::<f64>())
            .collect::<Result<Vec<f64>, std::num::ParseFloatError>>()
            .map_err(|_| "Unable to convert bbox coordinates to float")?;

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
