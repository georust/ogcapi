use anyhow::Ok;
use async_trait::async_trait;
use ogcapi_types::{
    common::{media_type::GEO_JSON, Crs},
    features::{Feature, FeatureCollection, Query},
};

use crate::FeatureTransactions;

use super::S3;

#[async_trait]
impl FeatureTransactions for S3 {
    async fn create_feature(&self, feature: &Feature) -> Result<String, anyhow::Error> {
        let key = format!(
            "collections/{}/items/{}.json",
            feature.collection.as_ref().unwrap(),
            feature.id.as_ref().unwrap()
        );
        let data = serde_json::to_vec(&feature)?;

        self.put_object("test-bucket", &key, data, Some(GEO_JSON.to_string()))
            .await?;

        Ok(key)
    }

    async fn read_feature(
        &self,
        collection: &str,
        id: &str,
        _crs: &Crs,
    ) -> Result<Feature, anyhow::Error> {
        let key = format!("collections/{}/items/{}.json", collection, id);

        let r = self.get_object("test-bucket", &key).await?;

        let c = serde_json::from_slice(&r.body.collect().await?.into_bytes()[..])?;

        Ok(c)
    }
    async fn update_feature(&self, feature: &Feature) -> Result<(), anyhow::Error> {
        let key = format!(
            "collections/{}/items/{}.json",
            feature.collection.as_ref().unwrap(),
            feature.id.as_ref().unwrap()
        );
        let data = serde_json::to_vec(&feature)?;

        self.put_object("test-bucket", &key, data, Some(GEO_JSON.to_string()))
            .await?;

        Ok(())
    }

    async fn delete_feature(&self, collection: &str, id: &str) -> Result<(), anyhow::Error> {
        let key = format!("collections/{}/items/{}.json", collection, id);

        self.delete_object("test-bucket", &key).await?;

        Ok(())
    }

    async fn list_items(
        &self,
        _collection: &str,
        _query: &Query,
    ) -> Result<FeatureCollection, anyhow::Error> {
        unimplemented!()
    }
}
