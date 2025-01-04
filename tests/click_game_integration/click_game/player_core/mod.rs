#![allow(dead_code)]

//module tree
mod core_plugin;
mod core_systems;
mod core_types;
mod handle_game_incoming;
mod handle_game_incoming_impl;
mod handle_player_inputs;
mod player_context;
mod player_initializer;
mod player_inputs;
mod setup;

//API exports
pub use core_plugin::*;
pub(crate) use core_systems::*;
pub use core_types::*;
pub(crate) use handle_game_incoming::*;
pub(crate) use handle_game_incoming_impl::*;
pub(crate) use handle_player_inputs::*;
pub use player_context::*;
pub use player_initializer::*;
pub use player_inputs::*;
pub(crate) use setup::*;
