use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Bbox {
    Bbox2D(f64, f64, f64, f64),
    Bbox3D(f64, f64, f64, f64, f64, f64),
}

impl std::fmt::Display for Bbox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Bbox::Bbox2D(lower_left_x, lower_left_y, upper_right_x, upper_right_y) => write!(
                f,
                "{},{},{},{}",
                lower_left_x, lower_left_y, upper_right_x, upper_right_y
            ),
            Bbox::Bbox3D(
                lower_left_x,
                lower_left_y,
                min_z,
                upper_right_x,
                upper_right_y,
                max_z,
            ) => {
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
