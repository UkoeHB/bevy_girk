//module tree
mod client_launchers;
mod client_monitor;
mod client_starter;
mod plugin;
mod types;

#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(target_family = "wasm")]
mod wasm;

//API exports
pub use crate::client_launchers::*;
pub use crate::client_monitor::*;
pub use crate::client_starter::*;
pub use crate::plugin::*;
pub use crate::types::*;

#[cfg(not(target_family = "wasm"))]
pub use crate::native::*;
#[cfg(target_family = "wasm")]
pub use crate::wasm::*;
