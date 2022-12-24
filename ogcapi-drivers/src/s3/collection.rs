use aws_sdk_s3::{error::GetObjectErrorKind, types::SdkError};

use ogcapi_types::common::{media_type::JSON, Collection, Collections, Query};

use crate::CollectionTransactions;

use super::S3;

#[async_trait::async_trait]
impl CollectionTransactions for S3 {
    async fn create_collection(&self, collection: &Collection) -> Result<String, anyhow::Error> {
        let key = format!("collections/{}/collection.json", collection.id);
        let data = serde_json::to_vec(&collection)?;

        self.put_object(
            self.bucket.clone().unwrap_or_default(),
            &key,
            data,
            Some(JSON.to_string()),
        )
        .await?;

        Ok(collection.id.to_owned())
    }

    async fn read_collection(&self, id: &str) -> Result<Option<Collection>, anyhow::Error> {
        // TODO: cache
        let key = format!("collections/{}/collection.json", id);

        match self
            .get_object(self.bucket.clone().unwrap_or_default(), &key)
            .await
        {
            Ok(r) => Ok(Some(serde_json::from_slice(
                &r.body.collect().await?.into_bytes(),
            )?)),
            Err(e) => match e {
                SdkError::ServiceError(err) => match err.err().kind {
                    GetObjectErrorKind::NoSuchKey(_) => Ok(None),
                    _ => Err(anyhow::Error::new(err.into_err())),
                },
                _ => Err(anyhow::Error::new(e)),
            },
        }
    }

    async fn update_collection(&self, collection: &Collection) -> Result<(), anyhow::Error> {
        let key = format!("collections/{}/collection.json", collection.id);
        let data = serde_json::to_vec(&collection)?;

        self.put_object(
            self.bucket.clone().unwrap_or_default(),
            &key,
            data,
            Some(JSON.to_string()),
        )
        .await?;

        Ok(())
    }

    async fn delete_collection(&self, id: &str) -> Result<(), anyhow::Error> {
        let key = format!("collections/{}", id);

        self.delete_object(self.bucket.clone().unwrap_or_default(), &key)
            .await?;

        Ok(())
    }

    async fn list_collections(&self, _query: &Query) -> Result<Collections, anyhow::Error> {
        let mut collections = Vec::new();

        let resp = self
            .client
            .list_objects()
            .bucket(self.bucket.clone().unwrap_or_default())
            .send()
            .await?;

        for object in resp.contents.unwrap() {
            if let Some(key) = object.key() {
                if key.ends_with("collection.json") {
                    let r = self
                        .get_object(self.bucket.clone().unwrap_or_default(), key)
                        .await?;

                    let c = serde_json::from_slice(&r.body.collect().await?.into_bytes()[..])?;

                    collections.push(c);
                }
            }
        }

        let mut collections = Collections::new(collections);
        collections.number_matched = collections.number_matched.to_owned();

        Ok(collections)
    }
}
