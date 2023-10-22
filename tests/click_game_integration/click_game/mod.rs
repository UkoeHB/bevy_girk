//module tree
mod game_core;
mod player_core;
mod wiring;

//API exports
pub(crate) use crate::click_game_integration::click_game::game_core::*;
pub(crate) use crate::click_game_integration::click_game::player_core::*;
pub(crate) use crate::click_game_integration::click_game::wiring::*;
