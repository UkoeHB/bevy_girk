//local shortcuts
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::*;
use bevy_replicon::prelude::*;

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

fn prestartup_check(world: &World)
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

#[bevy_plugin]
pub fn DummyGameCorePlugin(app: &mut App)
{
    // core request handler
    app.insert_resource(ClientRequestHandler::new(
            | _: &mut World, id: ClientId, packet: &ClientPacket | -> Result<(), Option<ClientFwRequest>>
            {
                deserialize_client_request(id, packet)
            }
        ));

    // startup check
    app.add_systems(PreStartup, prestartup_check);

    // game termination condition
    app.add_systems(PostUpdate, try_end_dummy_game);
}

//-------------------------------------------------------------------------------------------------------------------
