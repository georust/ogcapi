mod query;

pub use query::*;

use serde::{Deserialize, Serialize};

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
