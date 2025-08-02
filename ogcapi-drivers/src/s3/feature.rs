use aws_sdk_s3::{error::SdkError, operation::get_object::GetObjectError};

use ogcapi_types::{
    common::{Crs, media_type::GEO_JSON},
    features::{Feature, FeatureCollection, Query},
};

use crate::FeatureTransactions;

use super::S3;

#[async_trait::async_trait]
impl FeatureTransactions for S3 {
    async fn create_feature(&self, feature: &Feature) -> anyhow::Result<String> {
        let key = format!(
            "collections/{}/items/{}.json",
            feature.collection.as_ref().unwrap(),
            feature.id.as_ref().unwrap()
        );
        let data = serde_json::to_vec(&feature)?;

        self.put_object(
            self.bucket.clone().unwrap_or_default(),
            &key,
            data,
            Some(GEO_JSON.to_string()),
        )
        .await?;

        Ok(key)
    }

    async fn read_feature(
        &self,
        collection: &str,
        id: &str,
        _crs: &Crs,
    ) -> anyhow::Result<Option<Feature>> {
        let key = format!("collections/{collection}/items/{id}.json");

        match self
            .get_object(self.bucket.clone().unwrap_or_default(), &key)
            .await
        {
            Ok(r) => Ok(Some(serde_json::from_slice(
                &r.body.collect().await?.into_bytes(),
            )?)),
            Err(e) => match e {
                SdkError::ServiceError(err) => match err.err() {
                    GetObjectError::NoSuchKey(_) => Ok(None),
                    _ => Err(anyhow::Error::new(err.into_err())),
                },
                _ => Err(anyhow::Error::new(e)),
            },
        }
    }
    async fn update_feature(&self, feature: &Feature) -> anyhow::Result<()> {
        let key = format!(
            "collections/{}/items/{}.json",
            feature.collection.as_ref().unwrap(),
            feature.id.as_ref().unwrap()
        );
        let data = serde_json::to_vec(&feature)?;

        self.put_object(
            self.bucket.clone().unwrap_or_default(),
            &key,
            data,
            Some(GEO_JSON.to_string()),
        )
        .await?;

        Ok(())
    }

    async fn delete_feature(&self, collection: &str, id: &str) -> anyhow::Result<()> {
        let key = format!("collections/{collection}/items/{id}.json");

        self.delete_object(self.bucket.clone().unwrap_or_default(), &key)
            .await?;

        Ok(())
    }

    async fn list_items(
        &self,
        _collection: &str,
        _query: &Query,
    ) -> anyhow::Result<FeatureCollection> {
        unimplemented!()
    }
}
