//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::*;
use crate::test_helpers::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_attributes::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn test_game_tick_state(
    test_ctx   : Res<TestContext>,
    game_tick : Res<PlayTick>,
    mut flag   : ResMut<PanicOnDrop>,
){
    // expect the elapsed ticks match expected values
    if ***game_tick != test_ctx.num_game_ticks
    {
        panic!("game game tick state invalid: incorrect number of ticks elapsed");
    }

    flag.take();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
struct TestContext
{
    num_game_ticks: u32
}

fn test_game_ticks(num_players: usize, num_game_ticks: u32)
{
    let ticks_per_sec = 1;

    // run game until game over (no client fw or client core)
    App::new()
        //third-party plugins
        .add_plugins(bevy::time::TimePlugin)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .add_plugins(bevy_replicon::prelude::RepliconCorePlugin)
        .insert_resource(bevy_replicon::prelude::ReplicatedClients::new(VisibilityPolicy::All, true))
        .add_event::<bevy_replicon::prelude::StartReplication>()
        .add_plugins(VisibilityAttributesPlugin{
            server_id: Some(ClientId::SERVER),
            reconnect_policy: ReconnectPolicy::Reset
        })
        //setup app
        .set_runner(make_test_runner(num_game_ticks + 2))
        .add_plugins(AddMockMessageChannelsPlugin)
        //setup game framework
        .insert_resource(GameFwConfig::new( ticks_per_sec, 1, 0 ))
        .insert_resource(prepare_player_client_contexts(num_players))
        //setup game core
        .insert_resource(
                test_utils::prepare_game_initializer(
                        num_players,
                        GameDurationConfig::new(num_game_ticks),
                    )
            )
        //add game framework
        .add_plugins(GameFwPlugin)
        //add game
        .add_plugins(GamePlugins)
        //configure execution flow
        .configure_sets(Update, (GameFwSet::End,).chain())
        //testing
        .insert_resource(PanicOnDrop::default())
        .insert_resource( TestContext{ num_game_ticks } )
        .add_systems(OnEnter(GameState::GameOver), test_game_tick_state)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn game_ticks()
{
    test_game_ticks(1, 0);
    test_game_ticks(1, 0);
    test_game_ticks(1, 1);
    test_game_ticks(1, 2);
    test_game_ticks(2, 2);
}

//-------------------------------------------------------------------------------------------------------------------
