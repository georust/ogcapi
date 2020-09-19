use ogcapi::Service;
use tide::Result;

#[async_std::main]
async fn main() -> Result<()> {
    let database_url = "postgresql://postgres:postgres@localhost/ogcapi";
    let server_url = "http://192.168.1.218:8484";
    let service = Service::new(database_url).await;
    service.run(server_url).await?;
    Ok(())
}
