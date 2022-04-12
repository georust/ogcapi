use clap::Parser;
use ogcapi_drivers::postgres::Db;

#[derive(Parser, Debug)]
struct Config {
    /// Database url
    #[clap(env, hide_env_values = true, parse(try_from_str))]
    database_url: url::Url,
    /// Host address of the server
    #[clap(env, default_value = "0.0.0.0")]
    app_host: String,
    /// Port of the server
    #[clap(env, default_value = "8484")]
    app_port: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup env
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    // parse config
    let config = Config::parse();

    // setup database connection pool & run any pending migrations
    let db = Db::setup(&config.database_url).await?;

    // build application
    let router = ogcapi_services::server(db).await;

    // run our app with hyper
    let address = format!("{}:{}", config.app_host, config.app_port).parse()?;
    tracing::info!("listening on http://{}", address);

    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
