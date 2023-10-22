//! Miscellaneous systems.

//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_utils::*;
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
pub(crate) fn request_game_mode(mut client_message_buffer: ResMut<ClientMessageBuffer>)
{
    client_message_buffer.add_core_msg(&GameRequest::GameModeRequest, SendUnordered);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn send_game_request(In(msg): In<GameRequest>, mut client_message_buffer: ResMut<ClientMessageBuffer>)
{
    client_message_buffer.add_core_msg(&msg, SendOrdered);
}

//-------------------------------------------------------------------------------------------------------------------
