use std::net::SocketAddr;

use url::Url;
use uuid::Uuid;

use ogcapi_services::{AppState, Config, ConfigParser, Service};

#[allow(dead_code)]
pub async fn spawn_app() -> anyhow::Result<(SocketAddr, Url)> {
    dotenvy::dotenv().ok();

    // ogcapi_services::telemetry::init();

    let mut config = Config::parse();
    config.database_url.set_path(&Uuid::new_v4().to_string());
    config.port = 0;

    let state = AppState::new_from(&config).await;

    let service = Service::new_with(&config, state).await;

    let addr = service.local_addr()?;

    tokio::spawn(async move {
        service.serve().await;
    });

    Ok((addr, config.database_url))
}
