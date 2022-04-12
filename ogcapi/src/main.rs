use clap::Parser;
use ogcapi_drivers::postgres::Db;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[clap(name = "ogcapi_cli", version, about = "CLI for the ogcapi project.")]
pub struct App {
    /// Database url
    #[clap(long, parse(try_from_str), env, hide_env_values = true)]
    pub database_url: url::Url,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Imports geodata into the database
    #[cfg(feature = "import")]
    Import(ogcapi::import::Args),
    /// Starts the ogcapi services
    #[cfg(feature = "serve")]
    Serve {
        /// Host address of the server
        #[clap(env = "APP_HOST")]
        app_host: String,
        /// Port of the server
        #[clap(env = "APP_PORT")]
        app_port: String,
    },
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
    // tracing::debug!("{:#?}", app);

    match app.command {
        #[cfg(feature = "import")]
        Command::Import(args) => {
            // Import data
            if args.input.extension() == Some(std::ffi::OsStr::new("pbf")) {
                ogcapi::import::osm::load(args, &app.database_url).await?
            } else {
                ogcapi::import::ogr::load(args, &app.database_url).await?
            }
        }
        #[cfg(feature = "serve")]
        Command::Serve { app_host, app_port } => {
            // Setup a database connection pool & run any pending migrations
            let db = Db::setup(&app.database_url).await?;

            // Build our application
            let router = ogcapi_services::server(db).await;

            // run our app with hyper
            let address = format!("{}:{}", app_host, app_port).parse()?;
            tracing::info!("listening on http://{}", address);

            axum::Server::bind(&address)
                .serve(router.into_make_service())
                .await
                .unwrap();
        }
    }

    Ok(())
}
