use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum BBOX {
    XY(f64, f64, f64, f64),
    XYZ(f64, f64, f64, f64, f64, f64),
}

impl std::fmt::Display for BBOX {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BBOX::XY(lower_left_x, lower_left_y, upper_right_x, upper_right_y) => write!(
                f,
                "{},{},{},{}",
                lower_left_x, lower_left_y, upper_right_x, upper_right_y
            ),
            BBOX::XYZ(lower_left_x, lower_left_y, min_z, upper_right_x, upper_right_y, max_z) => {
                write!(
                    f,
                    "{},{},{},{},{},{}",
                    lower_left_x, lower_left_y, min_z, upper_right_x, upper_right_y, max_z
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bbox() {
        let s = "[ 160.6, -55.95, -170, -25.89 ]";
        let bbox: BBOX = serde_json::from_str(s).unwrap();

        match bbox {
            BBOX::XY { .. } => {}
            BBOX::XYZ { .. } => panic!("expected bbox to be 2 dimensional"),
        }
        assert_eq!(
            "[160.6,-55.95,-170.0,-25.89]",
            serde_json::to_string(&bbox).unwrap()
        );
    }
}
