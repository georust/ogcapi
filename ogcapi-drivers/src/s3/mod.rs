mod collection;
mod feature;

use aws_sdk_s3::{
    error::SdkError,
    operation::{
        delete_object::{DeleteObjectError, DeleteObjectOutput},
        get_object::{GetObjectError, GetObjectOutput},
        put_object::{PutObjectError, PutObjectOutput},
    },
    Client, Config,
};

pub use aws_sdk_s3::primitives::ByteStream;

/// S3 driver
#[derive(Clone)]
pub struct S3 {
    /// S3 client
    pub client: Client,
    /// Default bucket
    pub bucket: Option<String>,
}

impl S3 {
    pub async fn new() -> Self {
        let config = if let Ok(endpoint) = std::env::var("AWS_CUSTOM_ENDPOINT") {
            // Use custom enpoint if specified in `AWS_CUSTOM_ENDPOINT` environment variable
            println!("Setup client with custom endpoint: {endpoint}");
            aws_config::load_from_env()
                .await
                .into_builder()
                .endpoint_url(&endpoint)
                .build()
        } else {
            aws_config::load_from_env().await
        };

        // force path style addressing to work with minio
        let config = Config::from(&config)
            .to_builder()
            .force_path_style(true)
            .build();

        S3::new_with(Client::from_conf(config)).await
    }

    pub async fn new_with(client: Client) -> Self {
        S3 {
            client,
            bucket: None,
        }
    }

    pub fn set_default_bucket(&mut self, bucket: impl ToString) {
        self.bucket = Some(bucket.to_string())
    }

    pub async fn put_object(
        &self,
        bucket: impl Into<String>,
        key: impl Into<String>,
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
        bucket: impl Into<String>,
        key: impl Into<String>,
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
        bucket: impl Into<String>,
        key: impl Into<String>,
    ) -> Result<DeleteObjectOutput, SdkError<DeleteObjectError>> {
        self.client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
    }
}
