use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProcessQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
