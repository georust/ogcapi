#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup env
    ogcapi_services::setup_env();

    // setup tracing
    ogcapi_services::telemetry::init();

    // build & run our application with hyper
    ogcapi_services::Service::try_new().await?.serve().await;

    Ok(())
}
