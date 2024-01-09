//! Miscellaneous systems.

//local shortcuts
use bevy_girk_client_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Helper function-system for accessing the client core mode.
pub(crate) fn get_current_client_core_mode(current_client_core_mode: Res<State<ClientCoreMode>>) -> ClientCoreMode
{
    **current_client_core_mode
}

//-------------------------------------------------------------------------------------------------------------------

/// Request the current game mode.
pub(crate) fn request_game_mode(buffer: Res<ClientRequestBuffer>)
{
    buffer.request(GameRequest::GameModeRequest);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn send_game_request(In(msg): In<GameRequest>, buffer: Res<ClientRequestBuffer>)
{
    buffer.request(msg);
}

//-------------------------------------------------------------------------------------------------------------------
