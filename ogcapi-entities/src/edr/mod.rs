mod query;

pub use query::*;

pub use crate::common::Links;

use serde::{Deserialize, Serialize};

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
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct LandingPage {
    /// While a title is not required, implementers are strongly advised to include one.
    pub title: Option<String>,
    pub description: Option<String>,
    /// The `attribution` should be short and intended for presentation to a
    /// user, for example, in a corner of a map. Parts of the text can be links
    /// to other resources if additional information is needed. The string can
    /// include HTML markup.
    pub attribution: Option<String>,
    pub links: Links,
    pub keywords: Option<Vec<String>>,
    pub provider: Option<Provider>,
    pub contact: Option<Contact>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct Provider {
    /// Name of organization providing the service
    name: Option<String>,
    /// Link to service providers website
    url: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    email: Option<String>,
    phone: Option<String>,
    fax: Option<String>,
    hours: Option<String>,
    insructions: Option<String>,
    address: Option<String>,
    postal_code: Option<String>,
    city: Option<String>,
    stateorprovince: Option<String>,
    country: Option<String>,
}
