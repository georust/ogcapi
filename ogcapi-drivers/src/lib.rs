#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "s3")]
pub mod s3;

use ogcapi_types::{
    common::{Collection, Collections, Crs, Query as CollectionQuery},
    edr::{Query as EdrQuery, QueryType},
    features::{Feature, FeatureCollection, Query as FeatureQuery},
    processes::{Results, StatusInfo},
    styles::Styles,
    tiles::TileMatrixSet,
};
use serde_json::Value;

/// Trait for `Collection` transactions
#[async_trait::async_trait]
pub trait CollectionTransactions: Send + Sync {
    async fn create_collection(&self, collection: &Collection) -> anyhow::Result<String>;

    async fn read_collection(&self, id: &str) -> anyhow::Result<Option<Collection>>;

    async fn update_collection(&self, collection: &Collection) -> anyhow::Result<()>;

    async fn delete_collection(&self, id: &str) -> anyhow::Result<()>;

    async fn list_collections(&self, query: &CollectionQuery) -> anyhow::Result<Collections>;
}

/// Trait for `Feature` transactions
#[async_trait::async_trait]
pub trait FeatureTransactions: Send + Sync {
    async fn create_feature(&self, feature: &Feature) -> anyhow::Result<String>;

    async fn read_feature(
        &self,
        collection: &str,
        id: &str,
        crs: &Crs,
    ) -> anyhow::Result<Option<Feature>>;
    async fn update_feature(&self, feature: &Feature) -> anyhow::Result<()>;

    async fn delete_feature(&self, collection: &str, id: &str) -> anyhow::Result<()>;

    async fn list_items(
        &self,
        collection: &str,
        query: &FeatureQuery,
    ) -> anyhow::Result<FeatureCollection>;
}

/// Trait for `EDR` queries
#[async_trait::async_trait]
pub trait EdrQuerier: Send + Sync {
    async fn query(
        &self,
        collection_id: &str,
        query_type: &QueryType,
        query: &EdrQuery,
    ) -> anyhow::Result<FeatureCollection>;
}

/// Trait for `Processes` jobs
#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    async fn status(&self, id: &str) -> anyhow::Result<Option<StatusInfo>>;

    async fn delete(&self, id: &str) -> anyhow::Result<()>;

    async fn results(&self, id: &str) -> anyhow::Result<Option<Results>>;
}

/// Trait for `Style` transactions
#[async_trait::async_trait]
pub trait StyleTransactions: Send + Sync {
    async fn list_styles(&self) -> anyhow::Result<Styles>;

    async fn read_style(&self, id: &str) -> anyhow::Result<Option<Value>>;
}

/// Trait for `Tile` transacions
#[async_trait::async_trait]
pub trait TileTransactions: Send + Sync {
    async fn tile(
        &self,
        collections: &str,
        tms: &TileMatrixSet,
        matrix: &str,
        row: u32,
        col: u32,
    ) -> anyhow::Result<Vec<u8>>;
}
