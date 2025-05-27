use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::DisplayFromStr;

use crate::common::{Crs, Extent, Links};

// const CRS_REF: &str = "#/crs";

/// A body of resources that belong or are used together. An aggregate, set, or group of related resources.
#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    /// Must be set to `Collection` to be a valid Collection.
    #[cfg(feature = "stac")]
    #[serde(default = "collection")]
    pub r#type: String,
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    /// Attribution for the collection.
    pub attribution: Option<String>,
    pub extent: Option<Extent>,
    /// An indicator about the type of the items in the collection.
    #[cfg(not(feature = "movingfeatures"))]
    pub item_type: Option<String>,
    #[cfg(feature = "movingfeatures")]
    // TODO not sure if this is the best way to solve the requirement by moving features
    // to make itemType mandatory and still allowing to produce collections with other
    // itemTypes
    #[serde(flatten)]
    pub item_type: ItemType,
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
    /// Detailed information relevant to individual query types
    #[cfg(feature = "edr")]
    #[serde(rename = "data_queries")]
    pub data_queries: Option<crate::edr::DataQueries>,
    /// List of formats the results can be presented in
    #[cfg(feature = "edr")]
    #[serde(
        default,
        rename = "output_formats",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub output_formats: Vec<String>,
    /// List of the data parameters available in the collection
    #[cfg(feature = "edr")]
    #[serde(
        default,
        rename = "parameter_names",
        skip_serializing_if = "std::collections::HashMap::is_empty"
    )]
    pub parameter_names: std::collections::HashMap<String, crate::edr::ParameterNames>,
    /// The STAC version the Collection implements.
    #[cfg(feature = "stac")]
    #[serde(default = "crate::stac::stac_version", rename = "stac_version")]
    pub stac_version: String,
    // /// A list of extension identifiers the Collection implements.
    #[cfg(feature = "stac")]
    #[serde(
        default,
        rename = "stac_extensions",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub stac_extensions: Vec<String>,
    /// Collection's license(s), either a SPDX License identifier, `various` if
    /// multiple licenses apply or `proprietary` for all other cases.
    #[cfg(feature = "stac")]
    pub license: String,
    /// A list of providers, which may include all organizations capturing or processing the data or the hosting provider.
    #[cfg(feature = "stac")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<crate::stac::Provider>,
    /// A map of property summaries, either a set of values, a range of values or a JSON Schema.
    #[cfg(feature = "stac")]
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub summaries: Map<String, Value>,
    /// Dictionary of asset objects that can be downloaded, each with a unique key.
    #[cfg(feature = "stac")]
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub assets: std::collections::HashMap<String, crate::stac::Asset>,
    #[cfg(feature = "movingfeatures")]
    #[serde(rename = "updateFrequency")]
    /// A time interval of sampling location. The time unit of this property is millisecond.
    pub update_frequency: Option<i64>,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub additional_properties: Map<String, Value>,
}

#[cfg(feature = "stac")]
fn collection() -> String {
    "Collection".to_string()
}

#[cfg(feature = "movingfeatures")]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum ItemType {
    #[default]
    MovingFeature,
    Other(Option<String>)
}

#[allow(clippy::derivable_impls)]
impl Default for Collection {
    fn default() -> Self {
        Self {
            #[cfg(feature = "stac")]
            r#type: "Collection".to_string(),
            id: Default::default(),
            title: Default::default(),
            description: Default::default(),
            keywords: Default::default(),
            attribution: Default::default(),
            extent: Default::default(),
            item_type: Default::default(),
            crs: vec![Crs::default()],
            storage_crs: Default::default(),
            storage_crs_coordinate_epoch: Default::default(),
            links: Default::default(),
            #[cfg(feature = "edr")]
            data_queries: Default::default(),
            #[cfg(feature = "edr")]
            output_formats: Default::default(),
            #[cfg(feature = "edr")]
            parameter_names: Default::default(),
            #[cfg(feature = "stac")]
            stac_version: crate::stac::stac_version(),
            #[cfg(feature = "stac")]
            stac_extensions: Default::default(),
            #[cfg(feature = "stac")]
            license: "various".to_string(),
            #[cfg(feature = "stac")]
            providers: Default::default(),
            #[cfg(feature = "stac")]
            summaries: Default::default(),
            #[cfg(feature = "stac")]
            assets: Default::default(),
            #[cfg(feature = "movingfeatures")]
            update_frequency: Default::default(),
            additional_properties: Default::default(),
        }
    }
}
