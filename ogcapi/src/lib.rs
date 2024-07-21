#![doc = include_str!("../../README.md")]

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
