use futures::{StreamExt as _, TryStreamExt as _};
use object_store::{Error, ObjectStoreExt as _, PutMode, UpdateVersion, path::Path};

use ogcapi_types::common::{
    Collection, Collections, Exception, Link, Query,
    link_rel::{CHILD, PARENT, ROOT},
    media_type::JSON,
};

use crate::CollectionTransactions;

use super::{
    CATALOG, ObjectDriver, answered_links, collection_part, collection_path, invalid_id, not_found,
    stored_links,
};

#[async_trait::async_trait]
impl CollectionTransactions for ObjectDriver {
    async fn create_collection(&self, collection: &Collection) -> anyhow::Result<String> {
        let id = collection_part(&collection.id).ok_or_else(|| invalid_id(&collection.id))?;
        let payload = serde_json::to_vec(&stored(collection))?.into();
        if self
            .write(&collection_path(id), payload, PutMode::Create)
            .await?
            .is_none()
        {
            return Err(Exception::new_from_status(409)
                .detail(format!("a collection `{}` exists", collection.id))
                .into());
        }
        self.write_catalog().await?;
        Ok(collection.id.clone())
    }

    async fn read_collection(&self, id: &str) -> anyhow::Result<Option<Collection>> {
        let Some(id) = collection_part(id) else {
            return Ok(None);
        };
        let mut collection: Option<Collection> = self.read(&collection_path(id)).await?;
        if let Some(collection) = &mut collection {
            answered_links(&mut collection.links);
        }
        Ok(collection)
    }

    async fn update_collection(&self, collection: &Collection) -> anyhow::Result<()> {
        let location = collection_path(collection_part(&collection.id).ok_or_else(not_found)?);
        let _writes = self.writes.lock().await;
        if !self.exists(&location).await? {
            return Err(not_found());
        }
        self.store
            .put(&location, serde_json::to_vec(&stored(collection))?.into())
            .await?;
        Ok(())
    }

    async fn delete_collection(&self, id: &str) -> anyhow::Result<()> {
        let id = collection_part(id).ok_or_else(not_found)?;
        {
            let _writes = self.writes.lock().await;
            if !self.exists(&collection_path(id.clone())).await? {
                return Err(not_found());
            }
            // the collection's description, its features and its assets
            let locations = self
                .store
                .list(Some(&Path::from_iter([id])))
                .map_ok(|meta| meta.location)
                .boxed();
            self.store
                .delete_stream(locations)
                .try_collect::<Vec<_>>()
                .await?;
        }
        self.write_catalog().await
    }

    async fn list_collections(&self, _query: &Query) -> anyhow::Result<Collections> {
        let listed = self.store.list_with_delimiter(None).await?;
        let read: Vec<Option<Collection>> = futures::stream::iter(listed.common_prefixes)
            .filter(|prefix| std::future::ready(prefix.as_ref() != "jobs"))
            .map(|prefix| async move {
                self.read::<Collection>(&prefix.join("collection.json"))
                    .await
            })
            .buffered(16)
            .try_collect()
            .await?;
        // a collection whose description is gone, as it is deleted, is left out
        let mut collections: Vec<Collection> = read.into_iter().flatten().collect();
        collections.sort_by(|a, b| a.id.cmp(&b.id));
        for collection in &mut collections {
            answered_links(&mut collection.links);
        }

        let mut collections = Collections::new(collections);
        collections.number_matched = collections.number_returned;
        Ok(collections)
    }
}

impl ObjectDriver {
    /// Writes the root's catalog, linking every collection; written again
    /// where another write changed it meanwhile.
    async fn write_catalog(&self) -> anyhow::Result<()> {
        let location = Path::from(CATALOG);
        let _writes = self.writes.lock().await;
        for _ in 0..8 {
            let mode = match self.store.head(&location).await {
                Ok(meta) => PutMode::Update(UpdateVersion {
                    e_tag: meta.e_tag,
                    version: meta.version,
                }),
                Err(Error::NotFound { .. }) => PutMode::Create,
                Err(error) => return Err(error.into()),
            };
            let collections = self.list_collections(&Query::default()).await?;
            let links: Vec<Link> =
                std::iter::once(Link::new("./catalog.json", ROOT).mediatype(JSON))
                    .chain(collections.collections.iter().map(|collection| {
                        Link::new(
                            format!("./{}/collection.json", href_segment(&collection.id)),
                            CHILD,
                        )
                        .mediatype(JSON)
                    }))
                    .collect();
            let catalog = serde_json::json!({
                "type": "Catalog",
                "stac_version": "1.0.0",
                "id": "catalog",
                "description": "The collections of the store",
                "links": links,
            });
            let payload = serde_json::to_vec(&catalog)?.into();
            if self.write(&location, payload, mode).await?.is_some() {
                return Ok(());
            }
        }
        Err(Exception::new_from_status(503)
            .detail("the catalog changed under every write")
            .into())
    }
}

/// A collection as stored, linking the root's catalog as its root and parent.
fn stored(collection: &Collection) -> Collection {
    let catalog = |rel: &str| Link::new("../catalog.json", rel).mediatype(JSON);
    Collection {
        links: stored_links(&collection.links, [catalog(ROOT), catalog(PARENT)]),
        ..collection.clone()
    }
}

/// `id` as a segment of a relative reference's path, percent-encoded where
/// it isn't unreserved (RFC 3986).
fn href_segment(id: &str) -> String {
    id.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use ogcapi_types::common::{Collection, Exception, Link};

    use crate::CollectionTransactions;

    use super::ObjectDriver;

    fn status(error: &anyhow::Error) -> Option<u16> {
        error.downcast_ref::<Exception>().and_then(|e| e.status)
    }

    #[tokio::test]
    async fn collections_are_created_read_updated_listed_and_deleted() {
        let driver = ObjectDriver::default();

        let mut collection = Collection::new("a b.c");
        assert_eq!(
            driver.create_collection(&collection).await.unwrap(),
            "a b.c"
        );
        let error = driver.create_collection(&collection).await.unwrap_err();
        assert_eq!(status(&error), Some(409));
        driver
            .create_collection(&Collection::new("0"))
            .await
            .unwrap();

        collection.title = Some("title".to_string());
        driver.update_collection(&collection).await.unwrap();
        let read = driver.read_collection("a b.c").await.unwrap().unwrap();
        assert_eq!(read.title.as_deref(), Some("title"));

        let listed = driver.list_collections(&Default::default()).await.unwrap();
        let ids: Vec<&str> = listed.collections.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["0", "a b.c"]);
        assert_eq!(listed.number_matched, Some(2));

        driver.delete_collection("a b.c").await.unwrap();
        assert!(driver.read_collection("a b.c").await.unwrap().is_none());
        let error = driver.delete_collection("a b.c").await.unwrap_err();
        assert_eq!(status(&error), Some(404));
        let error = driver.update_collection(&collection).await.unwrap_err();
        assert_eq!(status(&error), Some(404));
    }

    #[tokio::test]
    async fn an_id_is_one_segment_of_a_path() {
        let driver = ObjectDriver::default();

        for id in ["", ".", "..", "a/b", "a\nb", "jobs", "catalog.json"] {
            let error = driver
                .create_collection(&Collection::new(id))
                .await
                .unwrap_err();
            assert_eq!(status(&error), Some(400), "{id:?}");
            assert!(driver.read_collection(id).await.unwrap().is_none());
        }

        // a collection's id is no prefix of another's
        driver
            .create_collection(&Collection::new("a"))
            .await
            .unwrap();
        driver
            .create_collection(&Collection::new("ab"))
            .await
            .unwrap();
        driver.delete_collection("a").await.unwrap();
        assert!(driver.read_collection("ab").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn the_store_is_a_static_catalog() {
        let driver = ObjectDriver::default();
        let mut collection = Collection::new("a b");
        collection.links = vec![
            Link::new("http://example.org/collections/a%20b", "self"),
            Link::new("http://example.org/license", "license"),
        ];
        driver.create_collection(&collection).await.unwrap();
        driver
            .create_collection(&Collection::new("c"))
            .await
            .unwrap();

        let catalog: serde_json::Value =
            driver.read(&"catalog.json".into()).await.unwrap().unwrap();
        let hrefs: Vec<&str> = catalog["links"]
            .as_array()
            .unwrap()
            .iter()
            .map(|link| link["href"].as_str().unwrap())
            .collect();
        assert_eq!(
            hrefs,
            [
                "./catalog.json",
                "./a%20b/collection.json",
                "./c/collection.json"
            ]
        );

        // stored with the catalog's links, relative, and answered without them
        let stored: Collection = driver
            .read(&"a b/collection.json".into())
            .await
            .unwrap()
            .unwrap();
        let links: Vec<(&str, &str)> = stored
            .links
            .iter()
            .map(|link| (link.rel.as_str(), link.href.as_str()))
            .collect();
        assert_eq!(
            links,
            [
                ("license", "http://example.org/license"),
                ("root", "../catalog.json"),
                ("parent", "../catalog.json")
            ]
        );
        let read = driver.read_collection("a b").await.unwrap().unwrap();
        assert_eq!(read.links.len(), 1);

        driver.delete_collection("c").await.unwrap();
        let catalog: serde_json::Value =
            driver.read(&"catalog.json".into()).await.unwrap().unwrap();
        assert_eq!(catalog["links"].as_array().unwrap().len(), 2);
    }
}
