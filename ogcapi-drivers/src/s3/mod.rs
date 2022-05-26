mod collection;
mod feature;

use aws_sdk_s3::{
    error::{DeleteObjectError, GetObjectError, PutObjectError},
    output::{DeleteObjectOutput, GetObjectOutput, PutObjectOutput},
    types::SdkError,
    Client, Endpoint,
};
use http::Uri;

pub use aws_sdk_s3::types::ByteStream;

pub struct S3 {
    pub client: Client,
}

impl S3 {
    pub async fn new() -> Self {
        let config = if let Ok(endpoint) = std::env::var("AWS_CUSTOM_ENDPOINT") {
            // Use custom enpoint if specified in `AWS_CUSTOM_ENDPOINT` environment variable
            let endpoint = endpoint.parse::<Uri>().unwrap();

            aws_config::from_env()
                .endpoint_resolver(Endpoint::immutable(endpoint))
                .load()
                .await
        } else {
            aws_config::from_env().load().await
        };

        S3::new_from(Client::new(&config)).await
    }

    pub async fn new_from(client: Client) -> Self {
        S3 { client }
    }

    pub async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(data))
            .set_content_type(content_type)
            .send()
            .await
    }

    pub async fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<GetObjectOutput, SdkError<GetObjectError>> {
        self.client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
    }

    pub async fn delete_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<DeleteObjectOutput, SdkError<DeleteObjectError>> {
        self.client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
    }
}
