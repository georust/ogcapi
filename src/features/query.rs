use std::fmt;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::common::{
    core::{Bbox, Datetime},
    Crs,
};

#[serde_as]
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Query {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub bbox: Option<Bbox>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bbox_crs: Option<Crs>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub datetime: Option<Datetime>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub crs: Option<Crs>,
    pub filter: Option<String>,
    pub filter_lang: Option<FilterLang>, // default = 'cql-text'
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub filter_crs: Option<Crs>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum FilterLang {
    CqlText,
    CqlJson,
}

impl Query {
    pub fn as_string_with_offset(&mut self, offset: i64) -> String {
        self.offset = Some(offset);
        self.to_string()
    }
}

impl fmt::Display for Query {
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
