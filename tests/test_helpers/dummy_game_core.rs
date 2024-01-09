//local shortcuts
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct DummyGameDurationConfig
{
    pub max_ticks: Ticks
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn prestartup_check(world: &World)
{
    if !world.contains_resource::<DummyGameDurationConfig>()
        { panic!("DummyGameDurationConfig is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_end_dummy_game(
    duration_config : Res<DummyGameDurationConfig>,
    game_fw_ticks   : Res<GameFWTicksElapsed>,
    mut end_flag    : ResMut<GameEndFlag>
){
    if duration_config.max_ticks > game_fw_ticks.elapsed.ticks() { return; }
    if end_flag.is_set() { return; }
    end_flag.set(GameOverReport::default());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[bevy_plugin]
pub fn DummyGameCorePlugin(app: &mut App)
{
    // core request handler
    app.insert_resource(ClientMessageHandler::new( | _: &mut World, _: &ClientPacket | -> bool { false } ));

    // startup check
    app.add_systems(PreStartup, prestartup_check);

    // game termination condition
    app.add_systems(PostUpdate, try_end_dummy_game);
}

//-------------------------------------------------------------------------------------------------------------------
