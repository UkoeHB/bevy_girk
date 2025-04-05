//module tree
mod basic_game_and_client;
mod basic_server_integration;
mod click_game;
mod game_instance_factory;
mod game_instance_launcher;
mod integration_reconnects;
mod player_clicks;
mod renet_minimal;
mod test_utils;

//API exports
pub(crate) use crate::click_game_integration::click_game::*;
pub(crate) use crate::click_game_integration::test_utils::*;
