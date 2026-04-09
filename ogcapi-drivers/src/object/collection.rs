use futures::StreamExt;
use object_store::{Error, ObjectStoreExt as _, PutMode, PutOptions, path::Path};

use ogcapi_types::common::{Collection, Collections, Query};

use crate::CollectionTransactions;

use super::ObjectDriver;

#[async_trait::async_trait]
impl CollectionTransactions for ObjectDriver {
    async fn create_collection(&self, collection: &Collection) -> Result<String, anyhow::Error> {
        let collection_id = collection.id.clone();

        let location = Path::from(format!("collections/{collection_id}/collection.json"));

        let payload = serde_json::to_vec(collection)?;

        let options = PutOptions {
            mode: PutMode::Create,
            ..Default::default()
        };
        self.store
            .put_opts(&location, payload.into(), options)
            .await?;

        Ok(collection_id.to_owned())
    }

    async fn read_collection(
        &self,
        collection_id: &str,
    ) -> Result<Option<Collection>, anyhow::Error> {
        let location = Path::from(format!("collections/{collection_id}/collection.json"));

        match self.store.get(&location).await {
            Ok(r) => {
                let v = r.bytes().await?;
                let collection = serde_json::from_slice(&v)?;
                Ok(Some(collection))
            }
            Err(e) => match e {
                Error::NotFound { path: _, source: _ } => Ok(None),
                _ => Err(anyhow::Error::new(e)),
            },
        }
    }

    async fn update_collection(&self, collection: &Collection) -> Result<(), anyhow::Error> {
        let collection_id = &collection.id;

        let location = Path::from(format!("collections/{collection_id}/collection.json"));

        let payload = serde_json::to_vec(collection)?;

        self.store.put(&location, payload.into()).await?;

        Ok(())
    }

    async fn delete_collection(&self, collection_id: &str) -> Result<(), anyhow::Error> {
        let prefix = Path::from(format!("collections/{collection_id}"));

        let mut list_stream = self.store.list(Some(&prefix));

        while let Some(result) = list_stream.next().await {
            let meta = result?;
            self.store.delete(&meta.location).await?;
        }

        self.store.delete(&prefix).await?;

        Ok(())
    }

    async fn list_collections(&self, _query: &Query) -> Result<Collections, anyhow::Error> {
        let prefix = Path::from("collections");

        let list_result = self.store.list_with_delimiter(Some(&prefix)).await?;

        let mut collections = Vec::with_capacity(list_result.common_prefixes.len());

        for path in list_result.common_prefixes {
            let collection = self
                .read_collection(path.parts().next_back().unwrap().as_ref())
                .await?;
            collections.push(collection.unwrap());
        }

        Ok(Collections::new(collections))
    }
}

#[cfg(test)]
mod tests {

    use ogcapi_types::common::Collection;

    use crate::CollectionTransactions;

    use super::*;

    #[tokio::test]
    async fn collection_crud() {
        let driver = ObjectDriver::default();

        // create collection
        let mut collection = Collection::new("test");

        let collection_id = driver.create_collection(&collection).await.unwrap();
        assert_eq!(collection_id, "test");

        // read collection
        let collection2 = driver.read_collection(&collection_id).await.unwrap();
        assert_eq!(collection2.as_ref(), Some(&collection));

        // update collection
        collection.title = Some("title".to_string());
        driver.update_collection(&collection).await.unwrap();

        let collection2 = driver.read_collection(&collection_id).await.unwrap();
        assert_eq!(collection2, Some(collection));

        // delete collection
        driver.delete_collection(&collection_id).await.unwrap();
        let collection2 = driver.read_collection(&collection_id).await.unwrap();
        assert!(collection2.is_none());
    }
}
