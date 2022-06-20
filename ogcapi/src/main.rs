use clap::Parser;

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

    // setup tracing
    ogcapi_services::telemetry::init();

    // parse cli args
    let app = App::parse();
    // tracing::debug!("{:#?}", app);

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
            // Application state
            let state = ogcapi_services::State::new_from(&config)
                .await
                .processors(vec![Box::new(ogcapi_services::Greeter)]);

            // Build & run with hyper
            ogcapi_services::Service::new_with(&config, state)
                .await
                .serve()
                .await;
        }
    }

    Ok(())
}
