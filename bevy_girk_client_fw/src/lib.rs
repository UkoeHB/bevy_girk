//documentation
#![doc = include_str!("../README.md")]

//module tree
mod client_fw_config;
mod client_request_sender;
mod game_message_handler;
mod handle_game_incoming;
mod handle_game_incoming_impl;
mod initialization_progress_cache;
mod ping_tracker;
mod plugin;
mod setup;
mod states;
mod systems;

//API exports
pub use client_fw_config::*;
pub use client_request_sender::*;
pub use game_message_handler::*;
pub(crate) use handle_game_incoming::*;
pub(crate) use handle_game_incoming_impl::*;
pub(crate) use initialization_progress_cache::*;
pub use ping_tracker::*;
pub use plugin::*;
pub(crate) use setup::*;
pub use states::*;
pub(crate) use systems::*;
