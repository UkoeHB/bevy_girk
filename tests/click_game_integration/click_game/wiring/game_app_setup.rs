//local shortcuts
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub fn prepare_game_app_core(game_app: &mut App, game_initializer: ClickGameInitializer)
{
    // depends on game framework

    // setup server with game core
    game_app
        .add_plugins(GamePlugins)
        .insert_resource(game_initializer);
}

//-------------------------------------------------------------------------------------------------------------------
