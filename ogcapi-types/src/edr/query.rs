use std::{
    fmt::{Display, Write},
    num::ParseFloatError,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use utoipa::{IntoParams, ToSchema};

use crate::common::{Crs, Datetime, Exception};

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum QueryType {
    Position,
    Radius,
    Area,
    Cube,
    Trajectory,
    Corridor,
    Items,
    Locations,
    Instances,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, IntoParams, Default, Debug)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Query {
    /// Well Known Text (WKT) of representation geometry. The representation
    /// type will depend on the [QueryType] of the API.
    #[serde(alias = "bbox")]
    pub coords: String,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[param(value_type = String)]
    pub datetime: Option<Datetime>,
    pub parameter_name: Option<String>,
    #[serde(default)]
    #[param(value_type = String)]
    pub crs: Option<Crs>,
    pub f: Option<String>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[param(value_type = String)]
    pub z: Option<Z>,
    pub within: Option<String>,
    pub within_units: Option<String>,
    pub resolution_x: Option<usize>,
    pub resolution_y: Option<usize>,
    pub resolution_z: Option<usize>,
    pub corridor_height: Option<String>,
    pub height_units: Option<String>,
    pub corridor_width: Option<String>,
    pub width_units: Option<String>,
}

/// The vertical level to return data for (available options are defined in the
/// vertical attribute of the extent section in the collections metadata response)
#[derive(Serialize, Deserialize, Debug)]
pub enum Z {
    Levels(Vec<f64>),
    Interval {
        start: f64,
        end: f64,
    },
    RInterval {
        repetitions: usize,
        interval: f64,
        start: f64,
    },
}

impl Display for Z {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Z::Levels(items) => {
                let mut iter = items.iter().peekable();
                while let Some(item) = iter.next() {
                    f.write_fmt(format_args!("{}", item))?;
                    if iter.peek().is_some() {
                        f.write_char(',')?;
                    }
                }
            }
            Z::Interval { start, end } => {
                if *start == f64::NEG_INFINITY {
                    f.write_str("../")?;
                } else {
                    f.write_fmt(format_args!("{}/", start))?;
                }

                if *end == f64::INFINITY {
                    f.write_str("..")?;
                } else {
                    f.write_fmt(format_args!("{}", end))?;
                }
            }
            Z::RInterval {
                repetitions,
                interval,
                start,
            } => {
                f.write_fmt(format_args!("R{repetitions}/{interval}/{start}"))?;
            }
        }

        Ok(())
    }
}

impl FromStr for Z {
    type Err = Exception;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('/') {
            let mut parts = s.split('/');

            // repetitions
            let repetitions = if s.starts_with('R') {
                Some(
                    parts
                        .next()
                        .unwrap()
                        .trim_start_matches('R')
                        .parse()
                        .map_err(|e| Exception::new_from_status(400).detail(e))?,
                )
            } else {
                None
            };

            // start/interval
            let part = parts
                .next()
                .ok_or(Exception::new_from_status(400).detail("malformed z parameter"))?;
            let first = if part == ".." {
                f64::NEG_INFINITY
            } else {
                part.parse()
                    .map_err(|e| Exception::new_from_status(400).detail(e))?
            };

            // end/start
            let part = parts
                .next()
                .ok_or(Exception::new_from_status(400).detail("malformed z parameter"))?;
            let second = if part == ".." {
                f64::INFINITY
            } else {
                part.parse()
                    .map_err(|e| Exception::new_from_status(400).detail(e))?
            };

            match repetitions {
                Some(repetitions) => Ok(Self::RInterval {
                    repetitions,
                    interval: first,
                    start: second,
                }),
                None => Ok(Self::Interval {
                    start: first,
                    end: second,
                }),
            }
        } else {
            let levels = s
                .split(',')
                .map(|p| p.parse())
                .collect::<Result<Vec<f64>, ParseFloatError>>()
                .map_err(|e| Exception::new_from_status(400).detail(e))?;
            Ok(Self::Levels(levels))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::edr::query::Z;

    #[test]
    fn z() {
        // Single level at level 850
        let z = "850";
        let zz = Z::from_str(z).unwrap().to_string();
        assert_eq!(z, zz);

        // All data between levels 100 and 550
        let z = "100/550";
        let zz = Z::from_str(z).unwrap().to_string();
        assert_eq!(z, zz);

        // All data between the minimum level and 850
        let z = "../850";
        let zz = Z::from_str(z).unwrap().to_string();
        assert_eq!(z, zz);

        // All data between the 500 and the maximum level
        let z = "500/..";
        let zz = Z::from_str(z).unwrap().to_string();
        assert_eq!(z, zz);

        // Data at levels 10,80,100
        let z = "10,80,200";
        let zz = Z::from_str(z).unwrap().to_string();
        assert_eq!(z, zz);

        // Data at 20 levels at 50 unit intervals starting a level 100
        let z = "R20/100/50";
        let zz = Z::from_str(z).unwrap().to_string();
        assert_eq!(z, zz);
    }
}
