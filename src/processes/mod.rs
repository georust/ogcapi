mod execute;
mod job;
mod process;

use std::fmt;

pub use execute::Execute;
pub use job::{Results, StatusCode, StatusInfo};
pub use process::{Process, ProcessSummary};

use serde::{Deserialize, Serialize};

use crate::common::core::Links;

/// Information about the available processes
#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessList {
    pub processes: Vec<ProcessSummary>,
    pub links: Links,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Query {
    pub(crate) limit: Option<i64>,
    pub(crate) offset: Option<i64>,
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
        write!(f, "{}", query_str.join("&"))
    }
}
