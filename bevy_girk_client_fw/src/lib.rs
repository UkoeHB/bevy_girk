//documentation
#![doc = include_str!("../README.md")]

//module tree
mod client_fw_config;
mod client_request_sender;
mod commands;
mod game_message_handler;
mod handle_commands;
mod handle_game_incoming;
mod handle_game_incoming_impl;
mod initialization_progress_cache;
mod ping_tracker;
mod plugin;
mod setup;
mod states;
mod systems;

//API exports
pub use crate::client_fw_config::*;
pub use crate::client_request_sender::*;
pub use crate::game_message_handler::*;
pub(crate) use crate::handle_commands::*;
pub(crate) use crate::handle_game_incoming::*;
pub(crate) use crate::handle_game_incoming_impl::*;
pub use crate::commands::*;
pub(crate) use crate::initialization_progress_cache::*;
pub use crate::ping_tracker::*;
pub use crate::plugin::*;
pub(crate) use crate::setup::*;
pub use crate::states::*;
pub(crate) use crate::systems::*;
