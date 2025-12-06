use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use utoipa::ToSchema;

use crate::common::{Bbox, Crs, Datetime};

#[serde_with::serde_as]
#[derive(Deserialize, ToSchema, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Query {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bbox: Option<Bbox>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = String)]
    pub bbox_crs: Option<Crs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Offset into the items list
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}
