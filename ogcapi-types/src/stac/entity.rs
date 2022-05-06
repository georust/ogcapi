use serde::{Deserialize, Serialize};

use crate::common::{Collection, Links};
use crate::features::Feature as Item;

use super::Catalog;

/// Type of STAC entity.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum StacEntity {
    Catalog(Catalog),
    Collection(Box<Collection>),
    Item(Item),
}

impl StacEntity {
    pub fn get_links_mut(&mut self) -> &mut Links {
        match self {
            StacEntity::Catalog(c) => &mut c.links,
            StacEntity::Collection(c) => &mut c.links,
            StacEntity::Item(i) => &mut i.links,
        }
    }
}
