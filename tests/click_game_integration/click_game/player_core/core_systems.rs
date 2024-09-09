//! Miscellaneous systems.

//local shortcuts
use bevy_girk_client_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Helper function-system for accessing the client core state.
pub(crate) fn get_current_client_core_state(current_client_core_state: Res<State<ClientCoreState>>) -> ClientCoreState
{
    **current_client_core_state
}

//-------------------------------------------------------------------------------------------------------------------

/// Request the current game state.
pub(crate) fn request_game_state(mut sender: ClientRequestSender)
{
    sender.request(GameRequest::GameStateRequest);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn send_game_request(In(msg): In<GameRequest>, mut sender: ClientRequestSender)
{
    sender.request(msg);
}

//-------------------------------------------------------------------------------------------------------------------
