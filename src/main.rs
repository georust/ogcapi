use anyhow::Result;
use ogcapi::import::{ogr, osm, Import};
use structopt::StructOpt;
use url::Url;

#[derive(StructOpt, Debug)]
#[structopt(name = "ogcapi", about = "A cli for the ogcapi project.")]
#[structopt(rename_all = "kebab-case")]
struct App {
    /// Database url
    #[structopt(parse(try_from_str), env, hide_env_values = true)]
    database_url: Url,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Imports geodata into the database
    Import(Import),
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
async fn main() -> Result<()> {
    // setup env
    dotenv::dotenv().ok();

    // read cli args
    let app = App::from_args();
    log::info!("{:?}", app);

    match app.command {
        Command::Import(args) => {
            // initialize logging
            env_logger::init();

            // Import data
            if args.input.extension() == Some(std::ffi::OsStr::new("pbf")) {
                osm::import(args, &app.database_url).await?
            } else {
                ogr::import(args, &app.database_url).await?
            }
        }
        Command::Serve { host, port } => {
            let _ = ogcapi::server::run(&host, &port, &app.database_url).await;
        }
    }

    Ok(())
}
