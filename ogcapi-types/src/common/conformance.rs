use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The Conformance declaration states the conformance classes from standards or community
/// specifications, identified by a URI, that the API conforms to. Clients can but are not
/// required to use this information. Accessing the Conformance declaration using HTTP GET
/// returns the list of URIs of conformance classes implemented by the server.
#[derive(Serialize, Deserialize, ToSchema, Default, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Conformance {
    pub conforms_to: Vec<String>,
}

impl Conformance {
    pub fn new(classes: &[impl ToString]) -> Self {
        Conformance {
            conforms_to: classes.iter().map(|c| c.to_string()).collect(),
        }
    }

    /// Extend conformance from other classes
    pub fn extend(&mut self, classes: &[impl ToString]) {
        self.conforms_to
            .extend(classes.iter().map(|c| c.to_string()))
    }
}
