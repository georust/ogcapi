use ogcapi::{
    processes::echo::Echo,
    services::{AppState, Config, ConfigParser, Drivers, Service},
};
use std::sync::Arc;

#[cfg(not(any(feature = "postgres", feature = "object")))]
compile_error!("the cite service needs a driver: `postgres` or `object`");

#[tokio::main]
async fn main() {
    // setup env
    ogcapi::services::setup_env();

    // setup tracing
    ogcapi::services::telemetry::init();

    // Config
    let config = Config::try_parse().unwrap();

    // Drivers
    #[cfg(feature = "postgres")]
    let drivers = Drivers::try_new_from_env().await.unwrap();
    // the object store at `OBJECT_STORE_URL`, in memory where it names none
    #[cfg(all(feature = "object", not(feature = "postgres")))]
    let drivers = {
        let store = ogcapi::drivers::object::ObjectDriver::from_env().unwrap();
        Drivers {
            collections: Box::new(store.clone()),
            features: Box::new(store.clone()),
            jobs: Box::new(store),
        }
    };

    // Application state
    let state = AppState::new(drivers).await;

    // Register processes/processors
    let state = state.processors(vec![Arc::new(Echo)]);

    // Build & run with hyper
    Service::try_new(&config, state)
        .await
        .unwrap()
        .all_apis()
        .serve()
        .await
        .expect("to serve application");
}
