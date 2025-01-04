//local shortcuts
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct DummyGameDurationConfig
{
    pub max_ticks: u32
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn build_precheck(world: &World)
{
    if !world.contains_resource::<DummyGameDurationConfig>()
    { panic!("DummyGameDurationConfig is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

// runs at the end of the current tick
fn try_end_dummy_game(
    duration_config : Res<DummyGameDurationConfig>,
    game_fw_tick    : Res<GameFwTick>,
    mut end_flag    : ResMut<GameEndFlag>
){
    if duration_config.max_ticks > ***game_fw_tick { return; }
    if end_flag.is_set() { return; }
    end_flag.set(GameOverReport::default());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub struct DummyGameCorePlugin;

impl Plugin for DummyGameCorePlugin
{
    fn build(&self, app: &mut App)
    {
        // core request handler
        app.insert_resource(ClientRequestHandler::new(|_, _, _: ()| {}));

        // startup check
        build_precheck(app.world());

        // game termination condition
        app.add_systems(PostUpdate, try_end_dummy_game);
    }
}

//-------------------------------------------------------------------------------------------------------------------
