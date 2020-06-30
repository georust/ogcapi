use async_std::task;
use ogcapi::features;
use tide::Result;

fn main() -> Result<()> {
    task::block_on(async {
        let api = "api/ogcapi-features-1.yaml";

        let db_url = "postgresql://postgres:postgres@localhost/ogcapi";

        let feature_service = features::Service::new(api, db_url).await;

        feature_service.run().await?;

        Ok(())
    })
}
