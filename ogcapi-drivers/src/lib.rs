#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "s3")]
pub mod s3;

#[cfg(feature = "common")]
use ogcapi_types::common::{Collection, Collections, Link, Query as CollectionQuery};
#[cfg(feature = "edr")]
use ogcapi_types::edr::{Query as EdrQuery, QueryType};
#[cfg(feature = "processes")]
use ogcapi_types::processes::{Response, StatusCode, StatusInfo};
#[cfg(feature = "stac")]
use ogcapi_types::stac::SearchParams;
#[cfg(feature = "styles")]
use ogcapi_types::styles::Styles;
#[cfg(feature = "tiles")]
use ogcapi_types::tiles::TileMatrixSet;
#[cfg(feature = "features")]
use ogcapi_types::{
    common::Crs,
    features::{Feature, Query as FeatureQuery},
};

#[cfg(any(feature = "features", feature = "stac", feature = "edr"))]
use ogcapi_types::features::FeatureCollection;

/// Trait for `Collection` transactions
#[cfg(feature = "common")]
#[async_trait::async_trait]
pub trait CollectionTransactions: Send + Sync {
    async fn create_collection(&self, collection: &Collection) -> anyhow::Result<String>;

    async fn read_collection(&self, id: &str) -> anyhow::Result<Option<Collection>>;

    async fn update_collection(&self, collection: &Collection) -> anyhow::Result<()>;

    async fn delete_collection(&self, id: &str) -> anyhow::Result<()>;

    async fn list_collections(&self, query: &CollectionQuery) -> anyhow::Result<Collections>;
}

/// Trait for `Feature` transactions
#[cfg(feature = "features")]
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

/// Trait for `STAC` search
#[cfg(feature = "stac")]
#[async_trait::async_trait]
pub trait StacSeach: Send + Sync {
    async fn search(&self, query: &SearchParams) -> anyhow::Result<FeatureCollection>;
}

/// Trait for `EDR` queries
#[cfg(feature = "edr")]
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
#[cfg(feature = "processes")]
#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    async fn register(&self, job: &StatusInfo, response_mode: Response) -> anyhow::Result<String>;

    async fn update(&self, job: &StatusInfo) -> anyhow::Result<()>;

    async fn status_list(&self, offset: usize, limit: usize) -> anyhow::Result<Vec<StatusInfo>>;

    async fn status(&self, id: &str) -> anyhow::Result<Option<StatusInfo>>;

    async fn finish(
        &self,
        job_id: &str,
        status: &StatusCode,
        message: Option<String>,
        links: Vec<Link>,
        results: Option<ogcapi_types::processes::ExecuteResults>,
    ) -> anyhow::Result<()>;

    async fn dismiss(&self, id: &str) -> anyhow::Result<Option<StatusInfo>>;

    async fn results(&self, id: &str) -> anyhow::Result<ProcessResult>;
}

#[cfg(feature = "processes")]
#[derive(Debug)]
pub enum ProcessResult {
    NoSuchJob,
    NotReady,
    Results {
        results: ogcapi_types::processes::ExecuteResults,
        response_mode: Response,
    },
}

/// Trait for `Style` transactions
#[cfg(feature = "styles")]
#[async_trait::async_trait]
pub trait StyleTransactions: Send + Sync {
    async fn list_styles(&self) -> anyhow::Result<Styles>;

    async fn read_style(&self, id: &str) -> anyhow::Result<Option<serde_json::Value>>;
}

/// Trait for `Tile` transacions
#[cfg(feature = "tiles")]
#[async_trait::async_trait]
pub trait TileTransactions: Send + Sync {
    async fn tile(
        &self,
        collections: &[String],
        tms: &TileMatrixSet,
        matrix: &str,
        row: u32,
        col: u32,
    ) -> anyhow::Result<Vec<u8>>;
}
