use ogcapi::Service;
use tide::Result;

#[async_std::main]
async fn main() -> Result<()> {
    // setup env
    dotenv::dotenv().ok();
    // let key = "DATABASE_URL";
    // env::set_var(key, "postgresql://postgres:postgres@localhost/ogcapi");

    let server_url = "http://192.168.1.218:8484";
    let service = Service::new().await;
    service.run(server_url).await?;
    Ok(())
}
