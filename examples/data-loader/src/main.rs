use clap::Parser;
use tracing_subscriber::{EnvFilter, prelude::*};

#[allow(unused_imports)]
use data_loader::{Args, Commands, is_geojson_file};

#[allow(unreachable_code, unused_variables)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup env
    dotenvy::dotenv().ok();

    // setup tracing
    tracing_subscriber::registry()
        .with(EnvFilter::new("data_loader=debug,ogcapi=debug,sqlx=warn"))
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    // parse cli args
    let args = Args::parse();
    tracing::debug!("{args:#?}");

    match args.command {
        #[cfg(feature = "client")]
        Commands::Client { public_url } => {
            if is_geojson_file(&args.input) && cfg!(feature = "geojson") {
                #[cfg(feature = "geojson")]
                data_loader::geojson::client::load(
                    args.input,
                    &args.collection,
                    args.s_srs,
                    &public_url,
                )
                .await?
            } else {
                tracing::warn!("Unsupported input, my try the `ogr` feature!")
            }
        }
        #[cfg(feature = "postgres")]
        Commands::Db { database_url } => {
            if is_geojson_file(&args.input) && cfg!(feature = "geojson") {
                #[cfg(feature = "geojson")]
                data_loader::geojson::db::load(
                    args.input,
                    &args.collection,
                    args.s_srs,
                    &database_url,
                )
                .await?
            } else if cfg!(feature = "ogr") {
                #[cfg(feature = "ogr")]
                data_loader::ogr::load(
                    args.input,
                    &args.collection,
                    &args.filter,
                    args.s_srs,
                    &database_url,
                )
                .await?
            }
        }
    }

    Ok(())
}
