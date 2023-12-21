//module tree
mod client_factory;
mod client_instance;
mod client_instance_command;
mod client_instance_launcher;
mod client_instance_report;
mod handle_command_incoming;
mod setup;

#[cfg(not(target_family = "wasm"))]
mod client_instance_launcher_local;
#[cfg(not(target_family = "wasm"))]
mod client_instance_launcher_process;

//API exports
pub use crate::client_factory::*;
pub use crate::client_instance::*;
pub use crate::client_instance_command::*;
pub use crate::client_instance_launcher::*;
pub use crate::client_instance_report::*;
pub(crate) use crate::handle_command_incoming::*;
pub use crate::setup::*;

#[cfg(not(target_family = "wasm"))]
pub use crate::client_instance_launcher_local::*;
#[cfg(not(target_family = "wasm"))]
pub use crate::client_instance_launcher_process::*;
