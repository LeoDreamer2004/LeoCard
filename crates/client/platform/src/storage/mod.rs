#[cfg(not(feature = "web"))]
mod native;
#[cfg(feature = "web")]
mod web;

#[cfg(not(feature = "web"))]
pub use native::*;
#[cfg(feature = "web")]
pub use web::*;
