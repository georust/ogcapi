use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use utoipa::ToSchema;

use crate::common::{Bbox, Crs, Datetime};

#[serde_with::serde_as]
#[derive(Deserialize, ToSchema, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Query {
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bbox: Option<Bbox>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = String)]
    pub bbox_crs: Option<Crs>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = String)]
    pub datetime: Option<Datetime>,
    #[serde(flatten)]
    pub pagination: LimitOffsetPagination,
    pub f: Option<String>,
}

/// Query parameters to facilitate pagination with a limit and offset
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
pub struct LimitOffsetPagination {
    /// Amount of items to return
    pub limit: Option<usize>,
    /// Offset into the items list
    pub offset: Option<usize>,
}
