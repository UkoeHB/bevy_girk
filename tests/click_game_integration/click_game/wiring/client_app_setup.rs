//local shortcuts
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Prepare the core of a click game client.
pub fn prepare_client_app_core(client_app: &mut App, player_initializer: ClickPlayerInitializer) -> Sender<PlayerInput>
{
    // depends on client framework

    // player input channel
    let (player_input_sender, player_input_receiver) = new_channel::<PlayerInput>();

    // make client app
    client_app
        .add_plugins(ClientPlugins)
        .insert_resource(player_initializer)
        .insert_resource(player_input_receiver);

    player_input_sender
}

//-------------------------------------------------------------------------------------------------------------------
