#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//module tree
mod cache_game_hubs;
mod cache_lobbies;
mod cache_ongoing_games;
mod cache_pending_lobbies;
mod cache_users;
mod channel_game_hub;
mod channel_user;
mod cleanup_handlers;
mod dc_buffer_game_hubs;
mod handle_game_hub_incoming;
mod handle_game_hub_incoming_impl;
mod handle_user_incoming;
mod handle_user_incoming_impl;
mod handler_utils;
mod server_setup;

//API exports
pub use crate::cache_game_hubs::*;
pub use crate::cache_lobbies::*;
pub use crate::cache_ongoing_games::*;
pub use crate::cache_pending_lobbies::*;
pub use crate::cache_users::*;
pub use crate::channel_game_hub::*;
pub use crate::channel_user::*;
pub(crate) use crate::cleanup_handlers::*;
pub use crate::dc_buffer_game_hubs::*;
pub(crate) use crate::handle_game_hub_incoming::*;
pub(crate) use crate::handle_game_hub_incoming_impl::*;
pub(crate) use crate::handle_user_incoming::*;
pub(crate) use crate::handle_user_incoming_impl::*;
pub(crate) use crate::handler_utils::*;
pub use crate::server_setup::*;
