mod data_queries;
mod observed_property;
mod parameter_names;
mod query;
mod units;

pub use data_queries::DataQueries;
pub use observed_property::ObservedPropertyCollection;
pub use parameter_names::ParameterNames;
pub use query::{Query, QueryType};
pub use units::Units;

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
    /// Email address of service provider
    email: Option<String>,
    /// Phone number of service provider
    phone: Option<String>,
    /// Fax number of service provider
    fax: Option<String>,
    hours: Option<String>,
    insructions: Option<String>,
    address: Option<String>,
    postal_code: Option<String>,
    city: Option<String>,
    stateorprovince: Option<String>,
    country: Option<String>,
}
