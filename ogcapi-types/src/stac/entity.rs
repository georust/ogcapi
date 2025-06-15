use serde::{Deserialize, Serialize};

use crate::common::{Collection, Link};
use crate::features::Feature as Item;

use super::Catalog;

/// Type of STAC entity.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum StacEntity {
    Catalog(Box<Catalog>),
    Collection(Box<Collection>),
    Item(Box<Item>),
}

impl StacEntity {
    pub fn get_links_mut(&mut self) -> &mut Vec<Link> {
        match self {
            StacEntity::Catalog(c) => &mut c.links,
            StacEntity::Collection(c) => &mut c.links,
            StacEntity::Item(i) => &mut i.links,
        }
    }
}
