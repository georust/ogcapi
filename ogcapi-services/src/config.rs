use clap::Parser;

#[derive(Parser, Debug)]
pub struct Config {
    /// Listening port of the server
    #[clap(long, env("APP_PORT"), default_value = "8484")]
    pub port: String,
    /// istening host address of the server
    #[clap(long, env("APP_HOST"), default_value = "0.0.0.0")]
    pub host: String,
    /// Postgres database url
    #[clap(long, env, hide_env_values = true, parse(try_from_str))]
    pub database_url: url::Url,
    /// OpenAPI definition
    #[clap(long, env, parse(from_os_str))]
    pub openapi: Option<std::path::PathBuf>,
}

pub fn parse_config() -> Config {
    Config::parse()
}
