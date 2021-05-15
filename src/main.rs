use std::path::PathBuf;

use anyhow::Result;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "ogcapi", about = "A cli for the ogcapi project.")]
enum Args {
    /// Imports geodata into the database
    Import {
        /// Input file
        #[structopt(parse(from_os_str))]
        input: PathBuf,

        /// Filter input by layer name or osm filter query, imports all if not present
        #[structopt(long)]
        filter: Option<String>,

        /// Set the collection name, defaults to layer name or "osm"
        #[structopt(long)]
        collection: Option<String>,
    },
    /// Starts the ogcapi service
    Serve {
        /// Host address the server listens to, defaults to env OGCAPI_HOST
        #[structopt(long, short)]
        host: Option<String>,

        /// Port the server listens to, defaults to env OGCAPI_PORT
        #[structopt(long, short)]
        port: Option<String>,
    },
}

#[async_std::main]
async fn main() -> Result<()> {
    // setup env
    dotenv::dotenv().ok();

    // initialize logging
    // env_logger::init();

    // read cli args
    let args = Args::from_args();
    log::info!("{:?}", args);

    match args {
        Args::Import {
            input,
            filter,
            collection,
        } => {
            ogcapi::import::import(input, &filter, &collection).await?;
        }
        Args::Serve { host, port } => {
            // Retrieve server address
            let host = host
                .or(std::env::var("OGCAPI_HOST").ok())
                .expect("Retrieve `OGCAPI_HOST`");
            let port = port
                .or(std::env::var("OGCAPI_PORT").ok())
                .expect("Retrieve `OGCAPI_PORT`");

            let service = ogcapi::Service::new().await;
            let _s = service.run(&format!("{}:{}", host, port)).await;
        }
    }

    Ok(())
}
