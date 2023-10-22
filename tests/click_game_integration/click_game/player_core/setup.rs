//local shortcuts
use bevy_girk_client_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Initialize the client state.
pub(crate) fn setup_player_state(world: &mut World)
{
    let player_initializer = world.remove_resource::<ClickPlayerInitializer>().expect("initializer missing");
    world.insert_resource(player_initializer.player_context);
}

//-------------------------------------------------------------------------------------------------------------------

/// Initialize the game output handler.
pub(crate) fn setup_game_output_handler(mut commands: Commands)
{
    commands.insert_resource::<GameMessageHandler>(GameMessageHandler::new(try_handle_game_core_output));
}

//-------------------------------------------------------------------------------------------------------------------
