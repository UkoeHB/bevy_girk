//features
#![feature(hash_drain_filter)]  //todo: nightly/unstable, will change to extract_if

//module tree
mod cache_pending_games;
mod cache_running_games;
mod cleanup_handlers;
mod game_hub_capacity_tracker;
mod game_hub_commands;
mod handle_commands;
mod handle_host_incoming;
mod handle_instance_reports;
mod handle_launch_pack_reports;
mod server_setup;

//API exports
pub use crate::cache_pending_games::*;
pub use crate::cache_running_games::*;
pub(crate) use crate::cleanup_handlers::*;
pub(crate) use crate::game_hub_capacity_tracker::*;
pub use crate::game_hub_commands::*;
pub(crate) use crate::handle_commands::*;
pub(crate) use crate::handle_host_incoming::*;
pub(crate) use crate::handle_instance_reports::*;
pub(crate) use crate::handle_launch_pack_reports::*;
pub use crate::server_setup::*;
