use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ogcapi::{
    processes::echo::Echo,
    services::{AppState, Config, Drivers, Service},
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

    // Drivers
    let drivers = Drivers::try_new_from_env().await.unwrap();

    // Application state
    let state = AppState::new(drivers).await;

    // Register processes/processors
    let state = state.processors(vec![
        // Box::new(Greeter),
        // Box::new(GeoJsonLoader),
        // Box::new(GdalLoader),
        Box::new(Echo),
    ]);

    // Build & run with hyper
    Service::try_new_with(&config, state)
        .await
        .unwrap()
        .serve()
        .await;
}
