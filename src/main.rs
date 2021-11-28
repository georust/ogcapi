use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "ogcapi", about = "A cli for the ogcapi project.")]
#[structopt(rename_all = "kebab-case")]
struct App {
    /// Log level
    #[structopt(long, env, default_value = "INFO")]
    rust_log: String,
    /// Database url
    #[structopt(parse(try_from_str), env, hide_env_values = true)]
    database_url: url::Url,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[cfg(feature = "import")]
    /// Imports geodata into the database
    Import(ogcapi::import::Args),
    /// Starts the ogcapi services
    Serve {
        /// Host address the server listens to, defaults to env OGCAPI_HOST
        #[structopt(long, short, env = "OGCAPI_HOST", default_value = "0.0.0.0")]
        host: String,

        /// Port the server listens to, defaults to env OGCAPI_PORT
        #[structopt(long, short, env = "OGCAPI_PORT", default_value = "8485")]
        port: String,
    },
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    // setup env
    dotenv::dotenv().ok();

    // read cli args
    let app = App::from_args();
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
        Command::Serve { host, port } => {
            let app = ogcapi::server::server(&app.database_url).await;
            app.listen(&format!("{}:{}", host, port)).await?;
        }
    }

    Ok(())
}
