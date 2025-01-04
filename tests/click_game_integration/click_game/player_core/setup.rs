//local shortcuts
use bevy_girk_client_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn setup_player_state(world: &mut World)
{
    let player_initializer = world.remove_resource::<ClickPlayerInitializer>().expect("initializer missing");
    world.insert_resource(player_initializer.player_context);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn setup_client_fw_reqs(world: &mut World)
{
    world.insert_resource(GameMessageHandler::new(try_handle_game_core_output));
    world.insert_resource(ClientRequestType::new::<GameRequest>());
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn cleanup_at_game_end(world: &mut World)
{
    world.remove_resource::<ClickPlayerContext>();
}

//-------------------------------------------------------------------------------------------------------------------
