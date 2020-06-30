use async_std::task;
use ogcapi::features;
use tide::Result;

fn main() -> Result<()> {
    task::block_on(async {
        let db_url = "postgresql://postgres:postgres@localhost/ogcapi";

        let feature_service = features::Service::new(db_url).await;

        feature_service.run().await?;

        Ok(())
    })
}
