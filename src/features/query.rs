use std::{default, fmt};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::common::{
    core::{Bbox, Datetime},
    crs::Crs,
};

#[serde_as]
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct FeaturesQuery {
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

impl default::Default for FilterLang {
    fn default() -> Self {
        FilterLang::CqlText
    }
}

impl fmt::Display for FeaturesQuery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut query_str = vec![];
        if let Some(limit) = self.limit {
            query_str.push(format!("limit={}", limit));
        }
        if let Some(offset) = self.offset {
            query_str.push(format!("offset={}", offset));
        }
        if let Some(bbox) = &self.bbox {
            query_str.push(format!("bbox={}", bbox));
        }
        if let Some(bbox_crs) = &self.bbox_crs {
            query_str.push(format!("bboxCrs={}", bbox_crs));
        }
        if let Some(datetime) = &self.datetime {
            query_str.push(format!("datetime={}", datetime));
        }
        if let Some(crs) = &self.crs {
            query_str.push(format!("crs={}", crs));
        }
        if let Some(filter) = &self.filter {
            query_str.push(format!("filter={}", filter));
        }
        if let Some(filter_lang) = &self.filter_lang {
            query_str.push(format!(
                "filter-lang={}",
                serde_json::to_string(filter_lang).expect("Serialize filter lang")
            ));
        }
        if let Some(filter_crs) = &self.filter_crs {
            query_str.push(format!("filter-crs={}", filter_crs));
        }
        write!(f, "{}", query_str.join("&"))
    }
}
