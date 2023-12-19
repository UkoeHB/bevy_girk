//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;
use crate::click_game_integration::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;
use bevy_kot_utils::*;
use bevy_renet::renet::RenetClient;

//standard shortcuts
use std::net::Ipv6Addr;
use std::time::{SystemTime, UNIX_EPOCH};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn game_is_initialized(game_init_progress: Query<&GameInitProgress>) -> bool
{
    Readiness::new(game_init_progress.single().0).is_ready()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_player_scores_system(
    In((expected_num_players, expected_num_clicks)) : In<(u32, u32)>,
    players                                         : Query<&PlayerScore, With<PlayerId>>
){
    let mut player_count = 0u32;
    for player_score in &players
    {
        assert_eq!(expected_num_clicks, player_score.score());
        player_count += 1;
    }

    assert_eq!(expected_num_players, player_count);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_player_scores(app: &mut App, expected_num_players: u32, expected_num_clicks: u32)
{
    syscall(&mut app.world, (expected_num_players, expected_num_clicks), check_player_scores_system);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn game_instance_factory_demo()
{
    // game info
    let num_players = 2;
    let num_watchers = 1;

    // config
    let ticks_per_sec   = Ticks(1);
    let max_init_ticks  = Ticks(200);
    let game_prep_ticks = Ticks(0);
    let game_play_ticks = Ticks(200);

    // server setup config
    let server_setup_config = GameServerSetupConfig{
            protocol_id     : get_test_protocol_id(),
            expire_seconds  : 10u64,
            timeout_seconds : 5i32,
            server_ip       : Ipv6Addr::LOCALHOST,
        };

    // game framework config
    let game_fw_config = GameFWConfig::new( ticks_per_sec, max_init_ticks, Ticks(0) );

    // game duration config
    let game_duration_config = GameDurationConfig::new(game_prep_ticks, game_play_ticks);

    // click game config
    let game_factory_config = ClickGameFactoryConfig{
            server_setup_config,
            game_fw_config,
            game_duration_config,
        };
    let game_factory_config_ser = ser_msg(&game_factory_config);

    // make game factory
    let game_factory = GameFactory::new(ClickGameFactory{});


    // make init data for the clients
    let mut client_init_data = Vec::<ClientInitDataForGame>::new();
    client_init_data.reserve(num_players + num_watchers);

    for i in 0..num_players
    {
        client_init_data.push(make_player_init_for_game(gen_rand128(), i as ClientIdType));
    }

    for i in num_players..(num_players + num_watchers)
    {
        client_init_data.push(make_watcher_init_for_game(gen_rand128(), i as ClientIdType));
    }


    // make new game
    let launch_pack = GameLaunchPack::new(0u64, game_factory_config_ser, client_init_data);
    let mut game_server_app = App::default();
    let mut game_start_report = game_factory.new_game(&mut game_server_app, launch_pack).unwrap();

    // get token meta
    let token_meta = game_start_report.native_meta.unwrap();

    // current time
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();

    // make clients
    let mut client_apps          = Vec::<App>::default();
    let mut player_input_senders = Vec::<Sender<PlayerInput>>::default();
    client_apps.reserve(num_players + num_watchers);
    player_input_senders.reserve(num_players);

    for start_info in game_start_report.start_infos.drain(..)
    {
        // make connect token
        let connect_token = new_connect_token_native(&token_meta, current_time, start_info.client_id).unwrap();

        // make client core
        // - we only make the core here, no client skin
        let (
                client_app,
                player_input_sender,
                _player_id
            ) = make_game_client_core(get_test_protocol_id(), connect_token, start_info);

        client_apps.push(client_app);
        if let Some(player_input_sender) = player_input_sender
        {
            player_input_senders.push(player_input_sender);
        }
    }


    // tick all the apps until initialized
    loop
    {
        // update server
        game_server_app.update();

        // expect that we have not left the init phase
        assert_eq!(*game_server_app.world.resource::<State<GameFWMode>>(), GameFWMode::Init);

        // update clients
        let mut num_inits = 0;

        for client in client_apps.iter_mut()
        {
            client.update();

            if *client.world.resource::<State<ClientInitializationState>>() != ClientInitializationState::Done
            { continue; }

            assert!(client.world.resource::<RenetClient>().is_connected());
            num_inits += 1;
        }

        // if not all clients are ready then we need to update them again
        if num_inits == num_players + num_watchers { break; }
    }

    // check that we have left the init phase as expected
    std::thread::sleep(std::time::Duration::from_millis(15));
    game_server_app.update();  //one update to collect client inputs notifying completion
    game_server_app.update();  //one update to update the mode
    assert_eq!(*game_server_app.world.resource::<State<GameFWMode>>(), GameFWMode::Game);

    assert!(syscall(&mut game_server_app.world, (), game_is_initialized));
    for client in client_apps.iter_mut()
    {
        client.update();  //load game initialization progress entity changes
        assert!(syscall(&mut client.world, (), game_is_initialized));
    }

    // send button clicks from each player
    for player_input_sender in player_input_senders.iter()
    {
        player_input_sender.send(PlayerInput::Play(PlayerInputPlay::ClickButton)).unwrap();
    }

    // tick until game over
    while *game_server_app.world.resource::<State<GameFWMode>>() != GameFWMode::End
    {
        game_server_app.update();

        for client in client_apps.iter_mut()
            { client.update(); }
    }


    // pull out watcher client
    let mut watcher_client_app = client_apps.pop().unwrap();

    // check test conditions
    check_player_scores(&mut game_server_app, num_players as u32, 1);

    for player_client in client_apps.iter_mut()
    {
        check_player_scores(player_client, num_players as u32, 1);
    }

    //todo: use information hiding to prevent watcher client from seeing scores (need bevy_replicon update)
    check_player_scores(&mut watcher_client_app, num_players as u32, 1);
}

//-------------------------------------------------------------------------------------------------------------------
