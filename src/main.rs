use std::{ffi::OsStr, path::PathBuf};

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "ogcapi", about = "A cli for the ogcapi project.")]
enum Args {
    /// Imports geodata into the database
    Import {
        /// Input file
        #[structopt(parse(from_os_str))]
        input: PathBuf,

        /// Specify the layer name to import, all if not present
        #[structopt(long)]
        layer: Option<String>,

        /// Filter osm input fatures by tags
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

    // read cli args
    let args = Args::from_args();
    println!("{:#?}", args);

    match args {
        Args::Import {
            input,
            layer,
            filter,
            collection,
        } => {
            // Create a connection pool
            let db_url = std::env::var("DATABASE_URL")?;
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&db_url)
                .await?;

            // Import data
            if input.extension() == Some(OsStr::new("pbf")) {
                ogcapi::import::osm_import(&input, &filter, &collection, &pool).await?
            } else {
                ogcapi::import::gdal_import(&input, &layer, &collection, &pool).await?;
            }
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
