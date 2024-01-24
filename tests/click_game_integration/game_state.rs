//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::*;
use crate::test_helpers::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn end_game(mut end_flag: ResMut<GameEndFlag>, mut flag: ResMut<PanicOnDrop>)
{
    end_flag.set(GameOverReport::default());
    flag.take();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn test_game_startup_state(num_players: Res<NumPlayers>, player_map: Res<PlayerMap>, players: Query<(Entity, &PlayerId)>)
{
    // count player entities
    let mut player_count = 0;
    for _ in &players { player_count += 1; }

    // expect the correct number of players are present
    if player_count != num_players.0
    {
        println!("num player entities: {}, expected: {}",
            player_count,
            num_players.0);
        panic!("game startup state invalid: incorrect num players");
    }

    // expect player map is accurate
    for (player_entity, player) in &players
    {
        let mapped_player_entity = player_map.client_to_entity(player.id).unwrap();
        if mapped_player_entity != player_entity { panic!("game startup state invalid: player id mismatch"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
struct NumPlayers(usize);

fn test_game_setup(num_players: usize)
{
    let ticks_per_sec = 1;

    // setup game (no client fw or client core)
    App::new()
        //bevy plugins
        .add_plugins(bevy::time::TimePlugin)
        .init_resource::<bevy_replicon::prelude::LastChangeTick>()
        //setup app
        .set_runner(make_test_runner(2))
        .add_plugins(AddMockMessageChannelsPlugin)
        //setup game framework
        .insert_resource(GameFwConfig::new( ticks_per_sec, 1, 0 ))
        .insert_resource(prepare_player_client_contexts(num_players))
        //setup game core
        .insert_resource(
                test_utils::prepare_game_initializer(
                        num_players,
                        GameDurationConfig::new(0, 0),
                    )
            )
        //add game framework
        .add_plugins(GameFwPlugin)
        //add game
        .add_plugins(GameStartupPlugin)
        .add_systems(Update, end_game.in_set(GameFwTickSet::End))
        //configure execution flow
        .configure_sets(Update, (GameFwSet,).chain())
        //testing
        .insert_resource(PanicOnDrop::default())
        .insert_resource(NumPlayers(num_players))
        .add_systems(Update, test_game_startup_state)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn game_state_setup()
{
    test_game_setup(1);
    test_game_setup(2);
}

//-------------------------------------------------------------------------------------------------------------------
