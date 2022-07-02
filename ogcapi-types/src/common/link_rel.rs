//! Link Relations
//!
//! [IANA Link Relations Registry](https://www.iana.org/assignments/link-relations/link-relations.xhtml)
//! [OGC Link Relation Type Register](http://www.opengis.net/def/rel)

pub const ABOUT: &str = "about";

/// Refers to a substitute for the link’s context.
pub const ATERNATE: &str = "alternate";

pub const CHILD: &str = "child";

pub const COLLECTION: &str = "collection";

/// Refers to a resource that identifies the specifications that the link’s context conforms to.
///
/// See: <http://www.opengis.net/def/rel/ogc/1.0/conformance>
pub const CONFORMANCE: &str = "conformance";

pub const DATA: &str = "data";

/// Identifies general metadata for the context (dataset or collection) that is primarily intended for consumption by machines.
///
/// See: <http://www.opengis.net/def/rel/ogc/1.0/data-meta>
pub const DATA_META: &str = "data-meta";

/// Refers to a resource providing information about the link’s context.
pub const DESCRIBEDBY: &str = "describedby";

/// The target URI points to exceptions of a failed process.
///
/// See: <http://www.opengis.net/def/rel/ogc/1.0/exceptions>
pub const EXCEPTIONS: &str = "exceptions";

/// The target URI points to the execution endpoint of the server.
///
/// See: <http://www.opengis.net/def/rel/ogc/1.0/execute>
pub const EXECUTE: &str = "execute";

pub const FIRST: &str = "first";

pub const ITEM: &str = "item";

pub const ITEMS: &str = "items";

/// The target URI points to the list of jobs.
///
/// See: <http://www.opengis.net/def/rel/ogc/1.0/job-list>
pub const JOB_LIST: &str = "job-list";

pub const LAST: &str = "last";

/// Refers to a license associated with the link’s context.
pub const LICENSE: &str = "license";

pub const METADATA: &str = "metadata";

pub const NEXT: &str = "next";

pub const PARENT: &str = "parent";

pub const PREV: &str = "prev";

/// The target URI points to the list of processes the API offers.
///
/// See: <http://www.opengis.net/def/rel/ogc/1.0/processes>
pub const PROCESSES: &str = "processes";

pub const RELATED: &str = "related";

/// The target URI points to the results of a job.
///
/// See: <http://www.opengis.net/def/rel/ogc/1.0/results>
pub const RESULTS: &str = "results";

pub const ROOT: &str = "root";

pub const SEARCH: &str = "search";

/// Conveys an identifier for the link’s context.
pub const SELF: &str = "self";

/// Identifies service description for the context that is primarily intended for consumption by machines.
pub const SERVICE_DESC: &str = "service-desc";

/// Identifies service documentation for the context that is primarily intended for human consumption.
pub const SERVICE_DOC: &str = "service-doc";

/// Identifies general metadata for the context that is primarily intended for consumption by machines.
pub const SERVICE_META: &str = "service-meta";

pub const START: &str = "start";

/// Identifies a resource that represents the context’s status.
pub const STATUS: &str = "status";

/// An asset that represents a thumbnail of the Item.
pub const THUMBNAIL: &str = "thumbnail";

pub const TILES: &str = "tiles";

/// The target IRI points to a resource that describes how to provide tile sets of the context resource in vector format.
///
/// See: <http://www.opengis.net/def/rel/ogc/1.0/tilesets-vector>
pub const TILESETS_VECTOR: &str = "tilesets-vector";

/// The target IRI points to a resource that describes the TileMatrixSet according to the 2D-TMS standard.
///
/// See: <http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme>
pub const TILING_SCHEME: &str = "tiling-scheme";

pub const OVERVIEW: &str = "overview";

/// Refers to a parent document in a hierarchy of documents.
pub const UP: &str = "up";
