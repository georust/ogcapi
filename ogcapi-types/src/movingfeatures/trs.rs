use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(tag = "type", content = "properties")]
pub enum Trs {
    Name {
        name: String,
    }, // r#type: String,
    Link {
        r#type: Option<String>,
        href: String,
    }, // r#type: String,
       // properties: TrsProperties,
}

impl Default for Trs {
    fn default() -> Self {
        Self::Name {
            name: "urn:ogc:data:time:iso8601".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn serde_json() {
        // TODO this contradicts example from https://developer.ogc.org/api/movingfeatures/index.html#tag/MovingFeatures/operation/retrieveMovingFeatures
        // Example from https://docs.ogc.org/is/19-045r3/19-045r3.html#_7_2_3_1_named_crs
        let trs: Trs = serde_json::from_str(
            r#"
            {
                "type": "Name",
                "properties": {"name": "urn:ogc:data:time:iso8601"}
            }
        "#,
        )
        .expect("Failed to parse Trs");
        let expected_trs = Trs::default();
        assert_eq!(trs, expected_trs);
    }
}
