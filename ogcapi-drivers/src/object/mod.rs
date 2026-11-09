//! A driver over an [`ObjectStore`], laid out as a static STAC catalog: the
//! root's `catalog.json` links each collection's `<id>/collection.json`, a
//! feature is `<collection>/<id>/<id>.json`, and a collection's asset
//! `<collection>/<name>`, each description linking its root and parent
//! relatively. A job is `jobs/<id>/status.json`, its response mode
//! `jobs/<id>/response.json` and its results `jobs/<id>/results.json`.

#[cfg(feature = "common")]
mod collection;
#[cfg(feature = "features")]
mod feature;
#[cfg(feature = "processes")]
mod job;

use std::sync::Arc;

use object_store::{
    Attribute, Attributes, Error, ObjectStore, ObjectStoreExt as _, PutMode, PutOptions,
    PutPayload, PutResult,
    local::LocalFileSystem,
    memory::InMemory,
    path::{Path, PathPart},
    prefix::PrefixStore,
};
use ogcapi_types::common::{
    Exception, Link,
    link_rel::{CHILD, COLLECTION, ITEM, ITEMS, PARENT, ROOT, SELF},
};
use url::Url;

/// The object store driver.
#[derive(Clone)]
pub struct ObjectDriver {
    store: Arc<dyn ObjectStore>,
    /// Held over a read and the write that depends on it, so that the
    /// driver's writes don't overtake each other where the store can't write
    /// on a condition, a local one.
    writes: Arc<futures::lock::Mutex<()>>,
}

impl ObjectDriver {
    /// A driver over `store`.
    pub fn new(store: impl ObjectStore) -> Self {
        Self {
            store: Arc::new(store),
            writes: Arc::default(),
        }
    }

    /// A driver over the store at `OBJECT_STORE_URL`, as [`Self::from_url`]
    /// takes it, in memory where it names none.
    pub fn from_env() -> anyhow::Result<Self> {
        match std::env::var("OBJECT_STORE_URL") {
            Ok(url) => Self::from_url(&url.parse()?),
            Err(std::env::VarError::NotPresent) => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    /// A driver over the store at `url`: `memory:///`, a store kept until
    /// the process ends; `file:///<directory>`, a directory of the local file
    /// system, created where it is missing; and with the feature `s3`
    /// `s3://<bucket>/<path>`, set up by the environment's `AWS_` variables
    /// as `object_store` names them (`AWS_ENDPOINT`, `AWS_ALLOW_HTTP`, ...).
    pub fn from_url(url: &Url) -> anyhow::Result<Self> {
        if url.scheme() == "file" {
            let directory = url
                .to_file_path()
                .map_err(|()| anyhow::anyhow!("{url} names no directory"))?;
            std::fs::create_dir_all(&directory)?;
            // a feature's directory goes with its description
            let store = LocalFileSystem::new_with_prefix(directory)?.with_automatic_cleanup(true);
            return Ok(Self::new(store));
        }
        let settings = std::env::vars().filter(|(key, _)| key.starts_with("AWS_"));
        let (store, prefix) = object_store::parse_url_opts(url, settings)?;
        Ok(if prefix.as_ref().is_empty() {
            Self {
                store: Arc::from(store),
                writes: Arc::default(),
            }
        } else {
            Self::new(PrefixStore::new(store, prefix))
        })
    }

    /// Stores `content` as the asset `name` of the collection `collection`,
    /// beside its description, of the media type `media_type` where it has
    /// one: the asset's path in the store; a refusal of an id or a name that
    /// can't be one segment of a path.
    pub async fn put_asset(
        &self,
        collection: &str,
        name: &str,
        content: Vec<u8>,
        media_type: Option<&str>,
    ) -> anyhow::Result<Path> {
        let collection_part = collection_part(collection).ok_or_else(|| invalid_id(collection))?;
        let name_part = item_part(name).ok_or_else(|| invalid_id(name))?;
        let location = Path::from_iter([collection_part, name_part]);
        let mut attributes = Attributes::new();
        if let Some(media_type) = media_type {
            attributes.insert(Attribute::ContentType, media_type.to_owned().into());
        }
        let options = PutOptions {
            attributes,
            ..Default::default()
        };
        let content = PutPayload::from(content);
        match self
            .store
            .put_opts(&location, content.clone(), options)
            .await
        {
            Ok(_) => {}
            // a store that keeps no content type, a file system's, keeps the
            // content alone: the asset's `type` says it
            Err(Error::NotImplemented { .. }) => {
                self.store.put(&location, content).await?;
            }
            Err(error) => return Err(error.into()),
        }
        Ok(location)
    }

    /// The object at `location` as JSON, `None` where there is none.
    async fn read<T: serde::de::DeserializeOwned>(
        &self,
        location: &Path,
    ) -> anyhow::Result<Option<T>> {
        match self.store.get(location).await {
            Ok(result) => Ok(Some(serde_json::from_slice(&result.bytes().await?)?)),
            Err(Error::NotFound { .. }) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Whether there is an object at `location`.
    async fn exists(&self, location: &Path) -> anyhow::Result<bool> {
        match self.store.head(location).await {
            Ok(_) => Ok(true),
            Err(Error::NotFound { .. }) => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    /// Writes `payload` at `location` in `mode`: what was written, `None`
    /// where the mode's condition doesn't hold, the object existing or
    /// changed. A store that can't write on a condition, a local one,
    /// overwrites.
    async fn write(
        &self,
        location: &Path,
        payload: PutPayload,
        mode: PutMode,
    ) -> anyhow::Result<Option<PutResult>> {
        match self
            .store
            .put_opts(location, payload.clone(), mode.into())
            .await
        {
            Ok(written) => Ok(Some(written)),
            Err(Error::AlreadyExists { .. } | Error::Precondition { .. }) => Ok(None),
            Err(Error::NotImplemented { .. }) => Ok(Some(self.store.put(location, payload).await?)),
            Err(error) => Err(error.into()),
        }
    }
}

/// A driver over a store in memory.
impl Default for ObjectDriver {
    fn default() -> Self {
        Self::new(InMemory::new())
    }
}

/// The root's catalog.
const CATALOG: &str = "catalog.json";

/// A collection's description.
const COLLECTION_FILE: &str = "collection.json";

/// `id` as one segment of a path; `None` for an id that is none: empty, `.`,
/// `..`, or holding a `/` or a control character.
fn segment(id: &str) -> Option<PathPart<'_>> {
    if id.is_empty() {
        return None;
    }
    PathPart::parse(id).ok()
}

/// A collection's segment, `None` where it is the catalog's or the jobs'.
fn collection_part(id: &str) -> Option<PathPart<'_>> {
    if matches!(id, CATALOG | "jobs") {
        return None;
    }
    segment(id)
}

/// A feature's or an asset's segment beside the collection's description,
/// `None` where it is the description's.
fn item_part(id: &str) -> Option<PathPart<'_>> {
    if id == COLLECTION_FILE {
        return None;
    }
    segment(id)
}

/// The path of the collection's description.
#[cfg(feature = "common")]
fn collection_path(collection: PathPart<'_>) -> Path {
    Path::from_iter([collection, PathPart::from(COLLECTION_FILE)])
}

/// The path of a feature; `None` where an id can't be a segment of one.
#[cfg(feature = "features")]
fn feature_path(collection: &str, feature: &str) -> Option<Path> {
    let collection = collection_part(collection)?;
    let directory = item_part(feature)?;
    let file = format!("{feature}.json");
    let file = PathPart::parse(&file).ok()?;
    Some(Path::from_iter([collection, directory, file]))
}

/// Relations of the API's structure and the static catalog's, which the
/// store sets itself.
const STRUCTURE: [&str; 7] = [SELF, ROOT, PARENT, CHILD, ITEM, ITEMS, COLLECTION];

/// `links` as stored: those of a structure left out, `structure`'s, the
/// static catalog's, added.
#[cfg(feature = "common")]
fn stored_links(links: &[Link], structure: impl IntoIterator<Item = Link>) -> Vec<Link> {
    links
        .iter()
        .filter(|link| !STRUCTURE.contains(&link.rel.as_str()))
        .cloned()
        .chain(structure)
        .collect()
}

/// `links` as answered: the static catalog's, relative, left out.
#[cfg(feature = "common")]
fn answered_links(links: &mut Vec<Link>) {
    links.retain(|link| Url::parse(&link.href).is_ok());
}

/// The refusal of an id that can't be one segment of a path.
fn invalid_id(id: &str) -> anyhow::Error {
    Exception::new_from_status(400)
        .detail(format!(
            "`{id}` is no id here: an id is not empty, `.` or `..`, holds no `/` or \
            control character, and is no name of the store's layout (`catalog.json` \
            and `jobs` for a collection, `collection.json` for a feature or an asset)"
        ))
        .into()
}

fn not_found() -> anyhow::Error {
    Exception::new_from_status(404).into()
}

#[cfg(test)]
mod tests {
    use object_store::{Attribute, ObjectStoreExt as _};

    use super::ObjectDriver;

    #[tokio::test]
    async fn an_asset_is_stored_with_its_media_type() {
        let driver = ObjectDriver::from_url(&"memory:///".parse().unwrap()).unwrap();
        let location = driver
            .put_asset(
                "places",
                "places.geojson",
                b"{}".to_vec(),
                Some("application/geo+json"),
            )
            .await
            .unwrap();
        assert_eq!(location.as_ref(), "places/places.geojson");
        let stored = driver.store.get(&location).await.unwrap();
        assert_eq!(
            stored
                .attributes
                .get(&Attribute::ContentType)
                .map(AsRef::as_ref),
            Some("application/geo+json")
        );
        assert_eq!(stored.bytes().await.unwrap().as_ref(), b"{}");
        for name in ["..", "collection.json"] {
            assert!(
                driver
                    .put_asset("places", name, Vec::new(), None)
                    .await
                    .is_err()
            );
        }
    }
}
