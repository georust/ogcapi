use clap::StructOpt;
use ogcapi::{
    cli::{App, Command},
    db::Db,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup env
    dotenv::dotenv().ok();

    // parse cli args
    let app = App::parse();
    if log::log_enabled!(log::Level::Info) {
        log::debug!("{:#?}", app);
    }

    match app.command {
        #[cfg(feature = "import")]
        Command::Import(args) => {
            // initialize logging
            env_logger::init();

            // Import data
            if args.input.extension() == Some(std::ffi::OsStr::new("pbf")) {
                ogcapi::import::osm::load(args, &app.database_url).await?
            } else {
                ogcapi::import::ogr::load(args, &app.database_url).await?
            }
        }
        #[cfg(feature = "server")]
        Command::Serve { app_host, app_port } => {
            // Set the RUST_LOG, if it hasn't been explicitly defined
            if std::env::var_os("RUST_LOG").is_none() {
                std::env::set_var("RUST_LOG", "api=debug,tower_http=debug")
            }
            tracing_subscriber::fmt::init();

            // Setup a database connection pool & run any pending migrations
            let db = Db::setup(&app.database_url).await?;

            // Build our application
            let router = ogcapi::server::server(db).await;

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
