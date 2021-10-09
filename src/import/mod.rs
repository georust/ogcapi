mod boundaries;
pub mod ogr;
pub mod osm;

use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Import {
    /// Input file
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,

    /// Filter input by layer name or osm filter query, imports all if not present
    #[structopt(long)]
    pub filter: Option<String>,

    /// Set the collection name, defaults to layer name or `osm`
    #[structopt(long)]
    pub collection: Option<String>,

    /// Source srs, defaults to the srs found in the input layer
    #[structopt(long)]
    pub s_srs: Option<u32>,

    /// Target storage crs of the collection
    #[structopt(long)]
    pub t_srs: Option<u32>,
}
