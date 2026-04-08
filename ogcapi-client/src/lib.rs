//! Client support for OGC APIs
//!
//! # Example
//!
//! ```rust,ignore
//! use futures_util::TryStreamExt;
//! use ogcapi_client::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::new("https://data.geo.admin.ch/api/stac/v0.9/").unwrap();
//!     let root = client.root().await.unwrap();
//!     println!("Root id: {}", root.id);
//!
//!     let mut collections = client.collections().await.unwrap();
//!     while let Some(c) = collections.try_next().await.unwrap() {
//!         println!("{}", c.id);
//!     }
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

pub use client::{Client, CollectionsStream, ItemsStream};
pub use error::Error;

#[cfg(feature = "blocking")]
#[cfg(not(target_arch = "wasm32"))]
pub use blocking::BlockingClient;
