use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Links;

/// The Landing page is the entry point of a OGC API
///
/// The Landing page provides links to:
///
/// * the API definition (link relations `service-desc` and `service-doc`),
///
/// * the Conformance declaration (path `/conformance`, link relation `conformance`), and
///
/// * the Collections (path `/collections`, link relation `data`).
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct LandingPage {
    /// The title of the API
    pub title: Option<String>,
    /// A textual description of the API
    pub description: Option<String>,
    /// The `attribution` should be short and intended for presentation to a
    /// user, for example, in a corner of a map. Parts of the text can be links
    /// to other resources if additional information is needed. The string can
    /// include HTML markup.
    pub attribution: Option<String>,
    /// Links to the resources exposed through this API
    pub links: Links,
    // pub keywords: Option<Vec<String>>,
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}
