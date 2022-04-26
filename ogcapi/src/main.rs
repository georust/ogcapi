use clap::Parser;
use ogcapi_drivers::postgres::Db;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[clap(name = "ogcapi", version, about = "CLI for the `ogcapi` project.")]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Import geodata into the database
    #[cfg(feature = "import")]
    Import(ogcapi::import::Args),
    /// Start the ogcapi services
    #[cfg(feature = "serve")]
    Serve(ogcapi_services::Config),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup env
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // parse cli args
    let app = App::parse();
    tracing::debug!("{:#?}", app);

    match app.command {
        #[cfg(feature = "import")]
        Command::Import(args) => {
            if let Some(extension) = args.input.extension() {
                match extension.to_str() {
                    Some("pbf") => ogcapi::import::osm::load(args).await?,
                    Some("geojson") => {
                        tracing::debug!("Using geojson loader ...");
                        ogcapi::import::geojson::load(args, true).await?
                    }
                    _ => ogcapi::import::ogr::load(args).await?,
                }
            }
        }
        #[cfg(feature = "serve")]
        Command::Serve(config) => {
            // Setup a database connection pool & run any pending migrations
            let db = Db::setup(&config.database_url).await?;

            // Build our application
            let router = ogcapi_services::app(db).await;

            // run our app with hyper
            let address = format!("{}:{}", config.host, config.port).parse()?;
            tracing::info!("listening on http://{}", address);

            axum::Server::bind(&address)
                .serve(router.into_make_service())
                .await
                .unwrap();
        }
    }

    Ok(())
}
