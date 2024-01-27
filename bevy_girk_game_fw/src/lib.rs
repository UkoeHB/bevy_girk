//documentation
#![doc = include_str!("../README.md")]

//module tree
mod basic_types;
mod client_fw_request;
mod client_readiness;
mod client_request_handler;
mod fw_types;
mod game_end_flag;
mod game_fw_clients;
mod game_fw_config;
mod game_fw_msg;
mod game_message_sender;
mod handle_requests;
mod handle_requests_impl;
mod packets;
mod plugin;
mod setup;
mod states;
mod systems;

//API exports
pub use crate::basic_types::*;
pub use crate::client_fw_request::*;
pub use crate::client_readiness::*;
pub use crate::client_request_handler::*;
pub use crate::fw_types::*;
pub use crate::game_end_flag::*;
pub use crate::game_fw_clients::*;
pub use crate::game_fw_config::*;
pub use crate::game_fw_msg::*;
pub use crate::game_message_sender::*;
pub(crate) use crate::handle_requests::*;
pub(crate) use crate::handle_requests_impl::*;
pub use crate::packets::*;
pub use crate::plugin::*;
pub(crate) use crate::setup::*;
pub use crate::states::*;
pub(crate) use crate::systems::*;
