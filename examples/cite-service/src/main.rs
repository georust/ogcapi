use ogcapi::{
    processes::echo::Echo,
    services::{AppState, Config, ConfigParser, Drivers, Service},
};

#[tokio::main]
async fn main() {
    // setup env
    ogcapi::services::setup_env();

    // setup tracing
    ogcapi::services::telemetry::init();

    // Config
    let config = Config::try_parse().unwrap();

    // Drivers
    let drivers = Drivers::try_new_from_env().await.unwrap();

    // Application state
    let state = AppState::new(drivers).await;

    // Register processes/processors
    let state = state.processors(vec![Box::new(Echo)]);

    // Build & run with hyper
    Service::try_new_with(&config, state)
        .await
        .unwrap()
        .all_apis()
        .serve()
        .await
        .expect("to serve application");
}
