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
pub use core_plugin::*;
pub(crate) use core_systems::*;
pub use core_types::*;
pub use game_context::*;
pub use game_duration_config::*;
pub use game_initializer::*;
pub use game_msg::*;
pub use game_over_report::*;
pub use game_request::*;
pub(crate) use handle_client_incoming::*;
pub(crate) use handle_client_incoming_impl::*;
pub use player_map::*;
pub use player_state::*;
pub use replication::*;
pub(crate) use setup::*;
pub use watcher_map::*;
