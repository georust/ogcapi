//! Client support for OGC APIs
//!
//! # Example
//!
//! ```rust, ignore
//! use ogcapi_client::Client;
//! use ogcapi_types::common::Bbox;
//! use ogcapi_types::stac::SearchParams;
//!
//! # fn main() {
//! // Setup a client for a given STAC endpoint
//! let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
//! let client = Client::new(endpoint).unwrap();
//!
//! // Fetch root catalog and print `id`
//! let root = client.root().unwrap();
//! println!("Root catalog id: {}", root.id);
//!
//! // Count catalogs
//! let catalogs = client.catalogs().unwrap();
//! println!("Found {} catalogs!", catalogs.count());
//!
//! // Search items
//! let bbox = Bbox::from([7.4473, 46.9479, 7.4475, 46.9481]);
//! let params = SearchParams::new()
//!     .with_bbox(bbox)
//!     .with_collections(["ch.swisstopo.swissalti3d"].as_slice());
//! let items = client.search(params).unwrap();
//! println!("Found {} items!", items.count());
//! # }

mod client;
mod error;

#[cfg(feature = "processes")]
mod processes;

pub use client::Client;
pub use error::Error;
