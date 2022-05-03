use clap::StructOpt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup env
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    // parse config
    let config = ogcapi_services::Config::parse();

    // setup database connection pool & run any pending migrations
    let db = ogcapi_drivers::postgres::Db::setup(&config.database_url).await?;

    // build application
    let router = ogcapi_services::app(db).await;

    // run our app with hyper
    let address = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("listening on http://{}", address);

    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
