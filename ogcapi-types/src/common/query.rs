use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;

use crate::common::{Bbox, Crs, Datetime};

#[serde_with::serde_as]
#[derive(Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Query {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bbox: Option<Bbox>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    pub bbox_crs: Option<Crs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    pub datetime: Option<Datetime>,
    #[serde(flatten)]
    pub pagination: LimitOffsetPagination,
    pub f: Option<String>,
}

/// Query parameters to facilitate pagination with a limit and offset
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct LimitOffsetPagination {
    /// Amount of items to return
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Offset into the items list
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}
