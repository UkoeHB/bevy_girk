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
pub use crate::click_game_integration::click_game::player_core::core_plugin::*;
pub(crate) use crate::click_game_integration::click_game::player_core::core_systems::*;
pub use crate::click_game_integration::click_game::player_core::core_types::*;
pub(crate) use crate::click_game_integration::click_game::player_core::handle_game_incoming::*;
pub(crate) use crate::click_game_integration::click_game::player_core::handle_game_incoming_impl::*;
pub(crate) use crate::click_game_integration::click_game::player_core::handle_player_inputs::*;
pub use crate::click_game_integration::click_game::player_core::player_context::*;
pub use crate::click_game_integration::click_game::player_core::player_initializer::*;
pub use crate::click_game_integration::click_game::player_core::player_inputs::*;
pub(crate) use crate::click_game_integration::click_game::player_core::setup::*;
