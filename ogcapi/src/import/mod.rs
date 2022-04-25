mod boundaries;
pub mod geojson;
pub mod ogr;
pub mod osm;

#[derive(clap::Parser, Default, Debug)]
pub struct Args {
    /// Input file
    #[clap(parse(from_os_str))]
    pub input: std::path::PathBuf,

    /// Set the collection name, defaults to layer name or `osm`
    #[clap(long)]
    pub collection: String,

    /// Filter input by layer name or osm filter query, imports all if not present
    #[clap(long)]
    pub filter: Option<String>,

    /// Source srs, defaults to the srs found in the input layer
    #[clap(long)]
    pub s_srs: Option<u32>,

    /// Target storage crs of the collection
    #[clap(long)]
    pub t_srs: Option<u32>,
}
