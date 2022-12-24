#[derive(clap::Parser, Debug)]
pub struct Args {
    /// Input file
    #[clap(long, value_parser)]
    pub input: std::path::PathBuf,

    /// Set the collection name, defaults to layer name or `osm`
    #[clap(long)]
    pub collection: String,

    /// Filter input by layer name or osm filter query, imports all if not present
    #[clap(long)]
    pub filter: Option<String>,

    /// Source srs, if omitted tries to derive from the input layer
    #[clap(long)]
    pub s_srs: Option<u32>,

    /// Target storage crs of the collection
    #[clap(long)]
    pub t_srs: Option<u32>,

    /// Postgres database url
    #[clap(long, env, hide_env_values = true, value_parser)]
    pub database_url: url::Url,
}

impl Args {
    pub fn new(
        input: impl Into<std::path::PathBuf>,
        collection: &str,
        database_url: &url::Url,
    ) -> Self {
        Args {
            input: input.into(),
            collection: collection.to_string(),
            filter: None,
            s_srs: None,
            t_srs: None,
            database_url: database_url.to_owned(),
        }
    }
}
