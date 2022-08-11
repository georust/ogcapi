#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup env
    dotenvy::dotenv().ok();

    // setup tracing
    ogcapi_services::telemetry::init();

    // build & run our application with hyper
    ogcapi_services::Service::new().await.serve().await;

    Ok(())
}
