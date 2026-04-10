use std::path::PathBuf;

use clap::{Parser, Subcommand};
use url::Url;

#[derive(Parser, Debug)]
pub struct Args {
    /// Input file
    #[clap(long, value_parser)]
    pub input: PathBuf,

    /// Set the collection name, defaults to layer name
    #[clap(long)]
    pub collection: String,

    /// Filter input by layer name, imports all if not present
    #[clap(long)]
    pub filter: Option<String>,

    /// Source srs, if omitted tries to derive from the input layer
    #[clap(long)]
    pub s_srs: Option<u32>,

    /// Loader, either `client` or `db` if features enabled
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Use `ogcapi-client` for ingestion
    #[cfg(feature = "client")]
    Client {
        /// Public OGC API root endpoint
        #[clap(long, env, hide_env_values = true, value_parser)]
        public_url: Url,
    },

    /// Directly bulk insert with the `Db` driver
    #[cfg(feature = "postgres")]
    Db {
        /// Postgres database url
        #[clap(long, env, hide_env_values = true, value_parser)]
        database_url: Url,
    },
}

impl Args {
    #[allow(unused)]
    pub fn new(input: impl Into<PathBuf>, collection: &str, url: &Url) -> Self {
        let command = if url.scheme().starts_with("http") {
            #[cfg(not(feature = "client"))]
            {
                tracing::info!("Need feature `client`");
                panic!("Need feature `client`");
            }

            #[cfg(feature = "client")]
            Commands::Client {
                public_url: url.to_owned(),
            }
        } else if url.scheme().starts_with("postgres") {
            #[cfg(not(feature = "postgres"))]
            {
                tracing::info!("Need feature `postgres`");
                panic!("Need feature `postgres`");
            }
            #[cfg(feature = "postgres")]
            Commands::Db {
                database_url: url.to_owned(),
            }
        } else {
            panic!("expect url to start with `http` or `postgres`");
        };

        Args {
            input: input.into(),
            collection: collection.to_string(),
            filter: None,
            s_srs: None,
            command,
        }
    }

    pub fn url(&self) -> &Url {
        #[allow(unreachable_patterns)]
        match &self.command {
            #[cfg(feature = "client")]
            Commands::Client { public_url } => public_url,
            #[cfg(feature = "postgres")]
            Commands::Db { database_url } => database_url,
            _ => todo!(),
        }
    }
}
