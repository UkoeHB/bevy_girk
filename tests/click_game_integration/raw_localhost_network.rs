//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;
use bevy_girk_wiring::*;
use crate::click_game_integration::*;
use crate::test_helpers::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_renet::renet::RenetClient;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn game_is_initialized(game_init_progress: Query<&GameInitProgress>) -> bool
{
    Readiness::new(**game_init_progress.single()).is_ready()
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

/// Collect clicks from clients and replicate them back out to the clients.
fn raw_localhost_network_demo(num_players: usize)
{
    // durations
    let ticks_per_sec   = 1;
    let max_init_ticks  = 200;
    let game_prep_ticks = 0;
    let game_play_ticks = 200;

    // game framework config
    let game_fw_config = GameFwConfig::new( ticks_per_sec, max_init_ticks, 0 );

    // game context
    let game_initializer = test_utils::prepare_game_initializer(
            num_players,
            GameDurationConfig::new(game_prep_ticks, game_play_ticks),
        );

    // prepare player clients
    let mut client_apps          = Vec::<App>::with_capacity(num_players);
    let mut player_input_senders = Vec::<Sender<PlayerInput>>::with_capacity(num_players);

    for client_id in 0..num_players
    {
        let mut client_app = App::new();

        // set up client framework
        let client_fw_config = ClientFwConfig::new(
                ticks_per_sec,
                ClientId::from_raw(client_id as u64),
            );

        let client_fw_command_sender = prepare_client_app_framework(&mut client_app, client_fw_config);
        client_app.add_plugins(bevy::time::TimePlugin);

        // set up replication
        prepare_client_app_replication(&mut client_app, client_fw_command_sender, std::time::Duration::from_millis(100));

        // prepare client initializer
        let player_context = ClickPlayerContext::new(
            ClientId::from_raw(client_id as u64),
                *game_initializer.game_context.duration_config()
            );
        let player_initializer = ClickPlayerInitializer{ player_context };
        let player_input_sender = prepare_client_app_core(&mut client_app, player_initializer);

        // save client app
        client_apps.push(client_app);
        player_input_senders.push(player_input_sender);
    }

    // make watcher client to demo non-player client with no replication
    let client_fw_config = ClientFwConfig::new(
            ticks_per_sec,
            ClientId::from_raw(num_players as u64),
        );

    let mut watcher_client_app = App::new();
    let client_fw_command_sender = prepare_client_app_framework(&mut watcher_client_app, client_fw_config);
    prepare_client_app_replication(&mut watcher_client_app, client_fw_command_sender, std::time::Duration::from_millis(100));
    watcher_client_app
        .add_plugins(bevy::time::TimePlugin)
        .add_plugins(DummyClientCorePlugin)
        .insert_resource(ClientRequestType::new::<GameRequest>())
        .add_plugins(GameReplicationPlugin);
    client_apps.push(watcher_client_app);

    // client initializers for server
    let mut game_fw_clients = prepare_player_client_contexts(num_players);
    append_client(&mut game_fw_clients);

    // prepare game server app
    let mut game_server_app = App::new();
    prepare_game_app_framework(&mut game_server_app, game_fw_clients, game_fw_config);
    prepare_game_app_replication(&mut game_server_app, std::time::Duration::from_millis(100), std::time::Duration::from_secs(10));
    prepare_game_app_core(&mut game_server_app, game_initializer);


    // make localhost test network connecting server to clients
    setup_local_test_renet_network(&mut game_server_app, &mut client_apps);


    // tick all the apps until initialized
    loop
    {
        // update server
        game_server_app.update();

        // expect that we have not left the init phase
        assert_eq!(*game_server_app.world.resource::<State<GameFwMode>>(), GameFwMode::Init);

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
        if num_inits == num_players + 1 { break; }
    }

    // check that we have left the init phase as expected
    std::thread::sleep(std::time::Duration::from_millis(15));
    game_server_app.update();  //one update to collect client inputs notifying completion
    game_server_app.update();  //one update to update the mode
    assert_eq!(*game_server_app.world.resource::<State<GameFwMode>>(), GameFwMode::Game);

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
    while *game_server_app.world.resource::<State<GameFwMode>>() != GameFwMode::End
    {
        game_server_app.update();

        for client in client_apps.iter_mut()
        {
            client.update();
        }
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
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn raw_localhost_network()
{
    /*
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env().unwrap()
        .add_directive("bevy_simplenet=trace".parse().unwrap())
        .add_directive("renet=trace".parse().unwrap())
        .add_directive("renetcode=trace".parse().unwrap())
        .add_directive("bevy_replicon=trace".parse().unwrap())
        .add_directive("bevy_girk_host_server=trace".parse().unwrap())
        .add_directive("bevy_girk_game_hub_server=trace".parse().unwrap())
        .add_directive("bevy_girk_wiring=trace".parse().unwrap())
        .add_directive("bevy_girk_demo_game_core=trace".parse().unwrap())
        .add_directive("bevy_girk_game_fw=trace".parse().unwrap());
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
    */
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    raw_localhost_network_demo(1usize);
    raw_localhost_network_demo(2usize);
    raw_localhost_network_demo(3usize);
}

//-------------------------------------------------------------------------------------------------------------------
