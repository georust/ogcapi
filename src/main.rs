use async_std::task;
use ogcapi::features;
use tide::Result;

fn main() -> Result<()> {
    task::block_on(async {
        let database_url = "postgresql://postgres:postgres@localhost/ogcapi";
        let server_url = "http://192.168.1.232:8484";

        features::service::run(server_url, database_url).await?;
        Ok(())
    })
}
