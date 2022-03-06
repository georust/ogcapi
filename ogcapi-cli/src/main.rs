use clap::StructOpt;
use ogcapi_cli::{App, Command};
use ogcapi_drivers::postgres::Db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup env
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    // parse cli args
    let app = App::parse();

    match app.command {
        #[cfg(feature = "import")]
        Command::Import(args) => {
            // Import data
            if args.input.extension() == Some(std::ffi::OsStr::new("pbf")) {
                ogcapi_cli::import::osm::load(args, &app.database_url).await?
            } else {
                ogcapi_cli::import::ogr::load(args, &app.database_url).await?
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
            tracing::info!("listening on {}", address);

            axum::Server::bind(&address)
                .serve(router.into_make_service())
                .await
                .unwrap();
        }
    }

    Ok(())
}
