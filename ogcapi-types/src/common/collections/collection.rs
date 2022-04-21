use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::common::{Crs, Extent, Links};
/// A body of resources that belong or are used together. An aggregate, set, or group of related resources.
#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    /// Must be set to `Collection` to be a valid Collection.
    #[cfg(feature = "stac")]
    pub r#type: String,
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    /// Attribution for the collection.
    pub attribution: Option<String>,
    pub extent: Option<Extent>,
    /// An indicator about the type of the items in the collection.
    pub item_type: Option<String>,
    /// The list of coordinate reference systems supported by the API; the first item is the default coordinate reference system.
    #[serde(default)]
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub crs: Vec<Crs>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub storage_crs: Option<Crs>,
    pub storage_crs_coordinate_epoch: Option<f32>,
    #[serde(default)]
    pub links: Links,
    /// The STAC version the Collection implements.
    #[cfg(feature = "stac")]
    pub stac_version: String,
    /// A list of extension identifiers the Collection implements.
    #[serde(default)]
    #[cfg(feature = "stac")]
    pub stac_extensions: Vec<String>,
    /// Collection's license(s), either a SPDX License identifier, `various` if
    /// multiple licenses apply or `proprietary` for all other cases.
    #[cfg(feature = "stac")]
    pub licence: String,
    /// A list of providers, which may include all organizations capturing or processing the data or the hosting provider.
    #[serde(default)]
    #[cfg(feature = "stac")]
    pub providers: Vec<crate::stac::Provider>,
    /// A map of property summaries, either a set of values, a range of values or a JSON Schema.
    #[serde(default)]
    #[cfg(feature = "stac")]
    pub summaries: std::collections::HashMap<String, serde_json::Value>,
    /// Dictionary of asset objects that can be downloaded, each with a unique key.
    #[serde(default)]
    #[cfg(feature = "stac")]
    pub assets: std::collections::HashMap<String, crate::stac::Asset>,
}
