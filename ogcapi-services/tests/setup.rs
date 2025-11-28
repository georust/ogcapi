use std::net::SocketAddr;

use url::Url;
use uuid::Uuid;

use ogcapi_services::{AppState, Config, ConfigParser, Drivers, Service};

#[allow(dead_code)]
pub async fn spawn_app() -> anyhow::Result<(SocketAddr, Url)> {
    dotenvy::dotenv().ok();

    // ogcapi_services::telemetry::init();

    let mut config = Config::parse();
    config.port = 0;

    let var = std::env::var("DATABASE_URL")?;
    let mut database_url = Url::parse(&var)?;
    database_url.set_path(&Uuid::new_v4().to_string());

    let drivers = Drivers::try_new_db(database_url.as_str()).await?;

    let state = AppState::new(drivers).await;

    let service = Service::try_new_with(&config, state).await?;

    let addr = service.local_addr()?;

    tokio::spawn(async move {
        service.serve().await;
    });

    Ok((addr, database_url))
}
