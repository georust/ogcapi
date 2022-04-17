use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::common::{Bbox, Crs, Datetime};

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Query {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bbox: Option<Bbox>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bbox_crs: Option<Crs>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub datetime: Option<Datetime>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub crs: Option<Crs>,
    pub filter: Option<String>,
    #[serde(default)]
    pub filter_lang: Option<FilterLang>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub filter_crs: Option<Crs>,
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