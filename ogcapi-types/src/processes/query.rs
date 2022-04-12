use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProcessQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
