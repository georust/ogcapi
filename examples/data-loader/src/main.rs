use clap::Parser;
use tracing_subscriber::{prelude::*, EnvFilter};

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
    let args = data_loader::Args::parse();
    tracing::debug!("{args:#?}");

    if let Some(extension) = args.input.extension() {
        #[allow(unreachable_patterns)]
        match extension.to_str() {
            #[cfg(feature = "geojson")]
            Some("geojson") => {
                tracing::info!("Using geojson loader ...");
                data_loader::geojson::load(args).await?
            }
            #[cfg(feature = "osm")]
            Some("pbf") => {
                tracing::info!("Using osm loader ...");
                data_loader::osm::load(args).await?
            }
            #[cfg(feature = "ogr")]
            _ => {
                tracing::info!("Using gdal loader ...");
                data_loader::ogr::load(args).await?
            }
            x => {
                tracing::warn!("No loader found for extension `{x:?}`! May need to activate additional features.");
            }
        }
    }

    Ok(())
}
