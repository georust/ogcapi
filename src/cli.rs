use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "ogcapi", version, about = "CLI for the ogcapi project.")]
pub struct App {
    /// Log level
    #[clap(env, default_value = "INFO")]
    pub rust_log: String,
    /// Database url
    #[clap(long, parse(try_from_str), env, hide_env_values = true)]
    pub database_url: url::Url,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Imports geodata into the database
    #[cfg(feature = "import")]
    Import(crate::import::Args),
    /// Starts the ogcapi services
    #[cfg(feature = "server")]
    Serve {
        /// Host address of the server
        #[clap(env = "APP_HOST")]
        app_host: String,
        /// Port of the server
        #[clap(env = "APP_PORT")]
        app_port: String,
    },
}
