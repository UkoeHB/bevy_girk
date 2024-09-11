//module tree
//todo: move to separate crate, constrain bevy_replicon features
mod client_app_setup;
//todo: move to separate crate, constrain bevy_replicon features
mod game_app_setup;
//todo: split to server/client crates
mod network_message_handling;
mod network_setup;
mod network_utils;

//API exports
pub use client_app_setup::*;
pub use game_app_setup::*;
pub(crate) use network_message_handling::*;
pub use network_setup::*;
pub use network_utils::*;
