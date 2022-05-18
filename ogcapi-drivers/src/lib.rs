#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "s3")]
pub mod s3;

use async_trait::async_trait;
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
#[async_trait]
pub trait CollectionTransactions: Send + Sync {
    async fn create_collection(&self, collection: &Collection) -> Result<String, anyhow::Error>;

    async fn read_collection(&self, id: &str) -> Result<Collection, anyhow::Error>;

    async fn update_collection(&self, collection: &Collection) -> Result<(), anyhow::Error>;

    async fn delete_collection(&self, id: &str) -> Result<(), anyhow::Error>;

    async fn list_collections(&self, query: &CollectionQuery)
        -> Result<Collections, anyhow::Error>;
}

/// Trait for `Feature` transactions
#[async_trait]
pub trait FeatureTransactions: Send + Sync {
    async fn create_feature(&self, feature: &Feature) -> Result<String, anyhow::Error>;

    async fn read_feature(
        &self,
        collection: &str,
        id: &str,
        crs: &Crs,
    ) -> Result<Feature, anyhow::Error>;
    async fn update_feature(&self, feature: &Feature) -> Result<(), anyhow::Error>;

    async fn delete_feature(&self, collection: &str, id: &str) -> Result<(), anyhow::Error>;

    async fn list_items(
        &self,
        collection: &str,
        query: &FeatureQuery,
    ) -> Result<FeatureCollection, anyhow::Error>;
}

/// Trait for `EDR` queries
#[async_trait]
pub trait EdrQuerier: Send + Sync {
    async fn query(
        &self,
        collection_id: &str,
        query_type: &QueryType,
        query: &EdrQuery,
    ) -> anyhow::Result<FeatureCollection>;
}

/// Trait for `Processes` jobs
#[async_trait]
pub trait JobHandler: Send + Sync {
    async fn status(&self, id: &str) -> Result<StatusInfo, anyhow::Error>;

    async fn delete(&self, id: &str) -> Result<(), anyhow::Error>;

    async fn results(&self, id: &str) -> Result<Results, anyhow::Error>;
}

/// Trait for `Style` transactions
#[async_trait]
pub trait StyleTransactions: Send + Sync {
    async fn list_styles(&self) -> Result<Styles, anyhow::Error>;

    async fn read_style(&self, id: &str) -> Result<Value, anyhow::Error>;
}

/// Trait for `Tile` transacions
#[async_trait]
pub trait TileTransactions: Send + Sync {
    async fn tile(
        &self,
        collections: &str,
        tms: &TileMatrixSet,
        matrix: &str,
        row: u32,
        col: u32,
    ) -> Result<Vec<u8>, anyhow::Error>;
}
