use async_std::task;
use ogcapi::features;
use tide::Result;

fn main() -> Result<()> {
    task::block_on(async {
        let api = "api/ogcapi-features-1.yaml";
        let db_url = "postgresql://postgres:postgres@localhost/ogcapi";

        features::service::run(api, db_url).await?;
        Ok(())
    })
}
