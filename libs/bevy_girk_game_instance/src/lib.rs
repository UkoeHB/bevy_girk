//module tree
mod game_factory;
mod game_instance;
mod game_instance_command;
mod game_instance_launcher;
mod game_instance_launcher_process;
mod game_instance_report;
mod handle_command_incoming;
mod setup;
mod types;

#[cfg(not(target_family = "wasm"))]
mod game_instance_launcher_local;

//API exports
pub use crate::game_factory::*;
pub use crate::game_instance::*;
pub use crate::game_instance_command::*;
pub use crate::game_instance_launcher::*;
pub use crate::game_instance_launcher_process::*;
pub use crate::game_instance_report::*;
pub(crate) use crate::handle_command_incoming::*;
pub use crate::setup::*;
pub use crate::types::*;

#[cfg(not(target_family = "wasm"))]
pub use crate::game_instance_launcher_local::*;
