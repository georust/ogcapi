mod collection;
mod feature;

use aws_sdk_s3::{
    error::{DeleteObjectError, GetObjectError, PutObjectError},
    output::{DeleteObjectOutput, GetObjectOutput, PutObjectOutput},
    types::SdkError,
    Client, Endpoint,
};
use aws_types::SdkConfig;
use http::Uri;

pub use aws_sdk_s3::types::ByteStream;

pub struct S3 {
    pub client: Client,
}

impl S3 {
    pub async fn setup() -> Self {
        let mut config = aws_config::from_env().load().await;

        // Use custom enpoint if specified in `AWS_CUSTOM_ENDPOINT` environment variable
        if let Ok(endpoint) = std::env::var("AWS_CUSTOM_ENDPOINT") {
            let endpoint = endpoint.parse::<Uri>().unwrap();

            let provider = config.credentials_provider().unwrap().to_owned();
            let region = config.region().unwrap().to_owned();

            config = SdkConfig::builder()
                .credentials_provider(provider)
                .region(region)
                .endpoint_resolver(Endpoint::immutable(endpoint))
                .build();
        }

        let client = Client::new(&config);

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