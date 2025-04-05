//! The `ogcapi` crate is organized in several modules:
//!
//! | Module / Crate    | Description     |
//! | ----------------- | --------------- |
//! | `types`    | Types as defined in various OGC API standards as well as STAC with `serde` support. |
//! | `client`   | Client to access HTTP endpoints of OGC API services as well as STAC wrapping `reqwest` |
//! | `services` | Server implementation of various OGC API services based on `axum`. |
//! | `drivers`  | Drivers for different data provider backends, currently mainly PostgreSQL with PostGIS through `sqlx`. |
//! | `processes` | `Processor` trait and implementations. |

#[cfg(feature = "client")]
pub mod client {
    pub use ogcapi_client::*;
}

#[cfg(feature = "drivers")]
pub mod drivers {
    pub use ogcapi_drivers::*;
}

#[cfg(feature = "processes")]
pub mod processes {
    pub use ogcapi_processes::*;
}

#[cfg(feature = "services")]
pub mod services {
    pub use ogcapi_services::*;
}

#[cfg(feature = "types")]
pub mod types {
    pub use ogcapi_types::*;
}
