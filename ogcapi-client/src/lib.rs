//! Client support for OGC APIs
//!
//! # Example
//!
//! ```rust,ignore
//! use ogcapi_client::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::new("https://data.geo.admin.ch/api/stac/v0.9/").unwrap();
//!     let root = client.root().await.unwrap();
//!     println!("Root id: {}", root.id);
//!
//!     let collections = client.all_collections().await.unwrap();
//!     println!("Found {} collections!", collections.len());
//! }
//! ```

mod client;
mod error;

#[cfg(feature = "blocking")]
#[cfg(not(target_arch = "wasm32"))]
mod blocking;

#[cfg(feature = "processes")]
pub mod processes;

pub(crate) static UA_STRING: &str = "OGCAPI-CLIENT";

pub use client::Client;
pub use error::Error;

#[cfg(feature = "blocking")]
#[cfg(not(target_arch = "wasm32"))]
pub use blocking::BlockingClient;
