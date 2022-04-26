mod asset;
mod catalog;
mod entity;
mod provider;
mod search;

pub use asset::Asset;
pub use catalog::Catalog;
pub use entity::StacEntity;
pub use provider::{Provider, ProviderRole};
pub use search::SearchParams;

#[doc(inline)]
pub use crate::common::Collection;

#[doc(inline)]
pub use crate::features::Feature as Item;

/// Default stac version
const STAC_VERSION: &str = "1.0.0";

pub(crate) fn stac_version() -> String {
    STAC_VERSION.to_owned()
}
