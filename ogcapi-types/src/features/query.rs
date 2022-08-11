use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;

use crate::common::{Bbox, Crs, Datetime};

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Query {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bbox: Option<Bbox>,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub bbox_crs: Crs,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub datetime: Option<Datetime>,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub crs: Crs,
    pub filter: Option<String>,
    #[serde(default)]
    pub filter_lang: Option<FilterLang>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub filter_crs: Option<Crs>,
    /// Parameters for filtering on feature properties
    #[serde(default, flatten)]
    pub additional_parameters: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum FilterLang {
    CqlText,
    CqlJson,
}

impl std::default::Default for FilterLang {
    fn default() -> Self {
        FilterLang::CqlText
    }
}
