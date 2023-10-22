#![allow(dead_code)]

//module tree
mod core_plugin;
mod core_systems;
mod core_types;
mod game_context;
mod game_duration_config;
mod game_initializer;
mod game_msg;
mod game_over_report;
mod game_request;
mod handle_client_incoming;
mod handle_client_incoming_impl;
mod misc_utils;
mod player_map;
mod player_state;
mod replication;
mod setup;
mod watcher_map;

//API exports
pub use crate::click_game_integration::click_game::game_core::core_plugin::*;
pub(crate) use crate::click_game_integration::click_game::game_core::core_systems::*;
pub use crate::click_game_integration::click_game::game_core::core_types::*;
pub use crate::click_game_integration::click_game::game_core::game_context::*;
pub use crate::click_game_integration::click_game::game_core::game_duration_config::*;
pub use crate::click_game_integration::click_game::game_core::game_initializer::*;
pub use crate::click_game_integration::click_game::game_core::game_msg::*;
pub use crate::click_game_integration::click_game::game_core::game_over_report::*;
pub use crate::click_game_integration::click_game::game_core::game_request::*;
pub(crate) use crate::click_game_integration::click_game::game_core::handle_client_incoming::*;
pub(crate) use crate::click_game_integration::click_game::game_core::handle_client_incoming_impl::*;
pub use crate::click_game_integration::click_game::game_core::misc_utils::*;
pub use crate::click_game_integration::click_game::game_core::player_map::*;
pub use crate::click_game_integration::click_game::game_core::player_state::*;
pub use crate::click_game_integration::click_game::game_core::replication::*;
pub(crate) use crate::click_game_integration::click_game::game_core::setup::*;
pub use crate::click_game_integration::click_game::game_core::watcher_map::*;
