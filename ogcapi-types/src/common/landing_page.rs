use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[cfg(feature = "edr")]
use crate::edr::{Contact, Provider};

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
    /// Set to `Catalog` if this Catalog only implements the Catalog spec.
    #[cfg(feature = "stac")]
    #[serde(default = "crate::stac::catalog")]
    pub r#type: String,
    /// The STAC version the Catalog implements.
    #[cfg(feature = "stac")]
    #[serde(default = "crate::stac::stac_version")]
    pub stac_version: String,
    /// A list of extension identifiers the Catalog implements.
    #[cfg(feature = "stac")]
    #[serde(default)]
    pub stac_extensions: Vec<String>,
    /// Identifier for the Catalog.
    #[cfg(feature = "stac")]
    pub id: String,
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
    #[serde(default)]
    pub links: Links,
    #[cfg(feature = "edr")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    #[cfg(feature = "edr")]
    pub provider: Option<Provider>,
    #[cfg(feature = "edr")]
    pub contact: Option<Contact>,
    #[cfg(feature = "stac")]
    pub conforms_to: Option<Vec<String>>,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub additional_properties: Map<String, Value>,
}
