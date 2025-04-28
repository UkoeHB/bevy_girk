//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_client_instance::*;
use bevy_girk_game_instance::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;
use crate::click_game_integration::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_renet2::prelude::RenetClient;
use renet2_setup::*;

//standard shortcuts
use std::net::Ipv6Addr;
use std::time::{SystemTime, UNIX_EPOCH};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn game_is_initialized(game_init_progress: Query<&GameInitProgress>) -> bool
{
    Readiness::new(**game_init_progress.single().unwrap()).is_ready()
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
    app.world_mut().syscall( (expected_num_players, expected_num_clicks), check_player_scores_system);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn game_instance_factory_demo()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // game info
    let num_players = 2;
    let num_watchers = 1;

    // config
    let ticks_per_sec   = 1;
    let max_init_ticks  = 50;
    let game_play_ticks = 200;

    // server setup config
    let server_setup_config = GameServerSetupConfig{
            protocol_id  : get_test_protocol_id(),
            expire_secs  : 10u64,
            timeout_secs : 5i32,
            server_ip    : Ipv6Addr::LOCALHOST.into(),
            native_port  : 0,
            wasm_wt_port : 0,
            wasm_ws_port : 0,
            proxy_ip     : None,
            wss_certs    : None,
            ws_domain    : None,
            native_port_proxy: 0,
            wasm_ws_port_proxy: 0,
            wasm_wt_port_proxy: 0,
            has_wss_proxy: false,
        };

    // game framework config
    let game_fw_config = GameFwConfig::new( ticks_per_sec, max_init_ticks, 0 );

    // game duration config
    let game_duration_config = GameDurationConfig::new(game_play_ticks);

    // click game config
    let game_factory_config = ClickGameFactoryConfig{
            server_setup_config,
            game_fw_config,
            game_duration_config,
        };

    // make game factory
    let game_factory = GameFactory::new(ClickGameFactory{});


    // make init data for the clients
    let mut client_init_data = Vec::<ClickClientInitDataForGame>::with_capacity(num_players + num_watchers);

    for i in 0..num_players
    {
        client_init_data.push(make_player_init_for_game(gen_rand128(), i as u64));
    }

    for i in num_players..(num_players + num_watchers)
    {
        client_init_data.push(make_watcher_init_for_game(gen_rand128(), i as u64));
    }


    // make new game
    let launch_pack = ClickLaunchPackData{ config: game_factory_config, clients: client_init_data};
    let launch_pack = GameLaunchPack::new(0u64, launch_pack);
    let mut game_server_app = App::default();
    let mut game_start_report = game_factory.new_game(&mut game_server_app, launch_pack).unwrap();

    // get token meta
    let token_meta = game_start_report.metas.native.unwrap();

    // current time
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();

    // make clients
    let mut client_apps          = Vec::<App>::with_capacity(num_players + num_watchers);
    let mut player_input_senders = Vec::<Sender<PlayerInput>>::with_capacity(num_players);

    for start_info in game_start_report.start_infos.drain(..)
    {
        // make connect token
        let connect_token = token_meta.new_connect_token(current_time, start_info.client_id).unwrap();

        // make client core
        // - we only make the core here, no client skin
        let mut client_factory = ClickClientFactory::new(get_test_protocol_id());
        let mut client_app = App::new();
        client_factory.add_plugins(&mut client_app);
        client_factory.setup_game(client_app.world_mut(), connect_token, ClientStartInfo::new(start_info).unwrap());
        let player_input_sender = client_factory.player_input.take();

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
        assert_eq!(*game_server_app.world().resource::<State<GameFwState>>(), GameFwState::Init);

        // update clients
        let mut num_inits = 0;

        for client in client_apps.iter_mut()
        {
            client.update();

            if *client.world().resource::<State<ClientInitState>>() != ClientInitState::Done
            { continue; }

            assert!(client.world().resource::<RenetClient>().is_connected());
            num_inits += 1;
        }

        // if not all clients are ready then we need to update them again
        if num_inits == num_players + num_watchers { break; }
    }

    // check that we have left the init phase as expected
    std::thread::sleep(std::time::Duration::from_millis(15));
    game_server_app.update();  //one update to collect client inputs notifying completion
    game_server_app.update();  //one update to update the state
    assert_eq!(*game_server_app.world().resource::<State<GameFwState>>(), GameFwState::Game);

    assert!(game_server_app.world_mut().syscall((), game_is_initialized));
    for client in client_apps.iter_mut()
    {
        client.update();  //load game initialization progress entity changes
        assert!(client.world_mut().syscall((), game_is_initialized));
    }

    // send button clicks from each player
    for player_input_sender in player_input_senders.iter()
    {
        player_input_sender.send(PlayerInput::Play(PlayerInputPlay::ClickButton)).unwrap();
    }

    // tick until game over
    while *game_server_app.world().resource::<State<GameFwState>>() != GameFwState::End
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
