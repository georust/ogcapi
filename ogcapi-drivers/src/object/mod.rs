#[cfg(feature = "common")]
mod collection;
#[cfg(feature = "features")]
mod feature;
#[cfg(feature = "processes")]
mod job;

use std::sync::Arc;

use object_store::{ObjectStore, memory::InMemory};
use url::Url;

/// Object store driver
#[derive(Clone)]
pub struct ObjectDriver {
    store: Arc<dyn ObjectStore>,
    #[allow(unused)]
    url: Url,
}

impl ObjectDriver {
    pub fn new(store: impl ObjectStore, url: Url) -> Self {
        let store = Arc::new(store);
        // TODO: remove url or test

        Self { store, url }
    }
}

impl Default for ObjectDriver {
    fn default() -> Self {
        Self::new(InMemory::new(), Url::parse("memory://").unwrap())
    }
}
