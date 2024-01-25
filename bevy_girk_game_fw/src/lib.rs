//documentation
#![doc = include_str!("../README.md")]

//module tree
mod client_fw_request;
mod client_readiness;
mod client_request_handler;
mod client_state;
mod game_end_flag;
mod game_fw_config;
mod game_fw_initializer;
mod game_fw_msg;
mod game_message_buffer;
mod handle_requests;
mod handle_requests_impl;
mod information_access;
mod packets;
mod plugin;
mod setup;
mod states;
mod systems;
mod types_common;
mod types_game_fw;

//API exports
pub use crate::client_fw_request::*;
pub use crate::client_readiness::*;
pub use crate::client_request_handler::*;
pub use crate::client_state::*;
pub use crate::game_end_flag::*;
pub use crate::game_fw_config::*;
pub use crate::game_fw_initializer::*;
pub use crate::game_fw_msg::*;
pub use crate::game_message_buffer::*;
pub(crate) use crate::handle_requests::*;
pub(crate) use crate::handle_requests_impl::*;
pub use crate::information_access::*;
pub use crate::packets::*;
pub use crate::plugin::*;
pub(crate) use crate::setup::*;
pub use crate::states::*;
pub(crate) use crate::systems::*;
pub use crate::types_common::*;
pub use crate::types_game_fw::*;
