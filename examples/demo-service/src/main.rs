use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ogcapi::{
    processes::{geojson_loader::GeoJsonLoader, greeter::Greeter},
    services::{AppState, Config, Service},
};

#[tokio::main]
async fn main() {
    // setup env
    dotenvy::dotenv().ok();

    // setup tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    // Config
    let config = Config::parse();

    // Application state
    let state = AppState::new_from(&config).await;

    // Register processes/processors
    let state = state.processors(vec![Box::new(Greeter), Box::new(GeoJsonLoader)]);

    // Build & run with hyper
    Service::new_with(&config, state).await.serve().await;
}
