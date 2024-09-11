//module tree
mod game_factory;
mod game_instance;
mod game_instance_command;
mod game_instance_launcher;
mod game_instance_report;
mod handle_command_incoming;
mod setup;
mod types;

#[cfg(not(target_family = "wasm"))]
mod game_instance_launcher_local_native;
#[cfg(not(target_family = "wasm"))]
mod game_instance_launcher_process;

#[cfg(target_family = "wasm")]
mod game_instance_launcher_local_wasm;

//API exports
pub use game_factory::*;
pub use game_instance::*;
pub use game_instance_command::*;
pub use game_instance_launcher::*;
pub use game_instance_report::*;
pub(crate) use handle_command_incoming::*;
pub use setup::*;
pub use types::*;

#[cfg(not(target_family = "wasm"))]
pub use game_instance_launcher_local_native::*;
#[cfg(not(target_family = "wasm"))]
pub use game_instance_launcher_process::*;

#[cfg(target_family = "wasm")]
pub use game_instance_launcher_local_wasm::*;
