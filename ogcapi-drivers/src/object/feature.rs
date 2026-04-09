use object_store::{Error, ObjectStoreExt, PutMode, PutOptions, path::Path};
use uuid::Uuid;

use ogcapi_types::{
    common::Crs,
    features::{Feature, FeatureCollection, FeatureId, Query},
};

use crate::FeatureTransactions;

use super::ObjectDriver;

#[async_trait::async_trait]
impl FeatureTransactions for ObjectDriver {
    async fn create_feature(&self, feature: &Feature) -> anyhow::Result<String> {
        let collection_id = feature.collection.as_ref().expect("collection id");

        // set random id if missing
        let mut feature_mut: Feature;
        let feature = if feature.id.is_none() {
            feature_mut = feature.to_owned();
            feature_mut.id = Some(FeatureId::String(Uuid::new_v4().to_string()));
            &feature_mut
        } else {
            feature
        };
        let feature_id = feature.id.as_ref().map(ToString::to_string).unwrap();

        let location = Path::from(format!(
            "collections/{collection_id}/items/{feature_id}.geojson"
        ));

        let payload = serde_json::to_vec(&feature)?;

        let options = PutOptions {
            mode: PutMode::Create,
            ..Default::default()
        };
        self.store
            .put_opts(&location, payload.into(), options)
            .await?;

        Ok(feature_id)
    }

    async fn read_feature(
        &self,
        collection_id: &str,
        feature_id: &str,
        _crs: &Crs,
    ) -> anyhow::Result<Option<Feature>> {
        let location = Path::from(format!(
            "collections/{collection_id}/items/{feature_id}.geojson"
        ));

        match self.store.get(&location).await {
            Ok(r) => Ok(Some(serde_json::from_slice(&r.bytes().await?)?)),
            Err(e) => match e {
                Error::NotFound { path: _, source: _ } => return Ok(None),
                _ => return Err(anyhow::Error::new(e)),
            },
        }
    }
    async fn update_feature(&self, feature: &Feature) -> anyhow::Result<()> {
        let collection_id = feature.collection.as_ref().expect("collection id");
        let feature_id = feature.id.as_ref().expect("feature id must be set");
        let location = Path::from(format!(
            "collections/{collection_id}/items/{feature_id}.geojson"
        ));

        let payload = serde_json::to_vec(&feature)?;

        self.store.put(&location, payload.into()).await?;

        Ok(())
    }

    async fn delete_feature(&self, collection_id: &str, feature_id: &str) -> anyhow::Result<()> {
        let location = Path::from(format!(
            "collections/{collection_id}/items/{feature_id}.geojson"
        ));

        self.store.delete(&location).await?;

        Ok(())
    }

    async fn list_items(
        &self,
        _collection_id: &str,
        _query: &Query,
    ) -> anyhow::Result<FeatureCollection> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {

    use ogcapi_types::{common::Collection, features::Geometry};

    use crate::CollectionTransactions;

    use super::*;

    #[tokio::test]
    async fn feature_crud() {
        let driver = ObjectDriver::default();

        // create collection
        let collection = Collection::new("test");

        let collection_id = driver.create_collection(&collection).await.unwrap();
        assert_eq!(collection_id, "test");

        // create feature
        let mut feature = Feature::new(Geometry::new_point([0.0, 0.0]));
        feature.collection = Some(collection_id.clone());
        let feature_id = driver.create_feature(&feature).await.unwrap();
        feature.id = Some(FeatureId::String(feature_id.clone()));

        // read feature
        let feature2 = driver
            .read_feature(&collection_id, &feature_id, &Crs::default2d())
            .await
            .unwrap();
        assert_eq!(feature2.as_ref(), Some(&feature));

        // update feature
        feature.properties.insert("key".to_string(), "value".into());
        driver.update_feature(&feature).await.unwrap();

        let feature2 = driver
            .read_feature(&collection_id, &feature_id, &Crs::default2d())
            .await
            .unwrap();
        assert_eq!(feature2, Some(feature));

        // delete feature
        driver
            .delete_feature(&collection_id, &feature_id)
            .await
            .unwrap();
        let feature2 = driver
            .read_feature(&collection_id, &feature_id, &Crs::default2d())
            .await
            .unwrap();
        assert!(feature2.is_none());
    }
}
