use clap::Parser;

/// Application configuration
#[derive(Parser, Debug)]
pub struct Config {
    /// Listening port of the server
    #[clap(long, env("APP_PORT"), default_value = "8484")]
    pub port: u16,
    /// listening host address of the server
    #[clap(long, env("APP_HOST"), default_value = "0.0.0.0")]
    pub host: String,
    /// Postgres database url
    #[clap(long, env, hide_env_values = true, value_parser)]
    pub database_url: url::Url,
    /// OpenAPI definition
    #[clap(long, env, value_parser)]
    pub openapi: Option<std::path::PathBuf>,
}
