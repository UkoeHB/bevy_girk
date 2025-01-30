//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_client_instance::*;
use bevy_girk_game_instance::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;
use bevy_girk_wiring_common::*;
use crate::click_game_integration::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_replicon::prelude::*;
use bevy_renet2::prelude::RenetClient;

//standard shortcuts
use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn game_is_initialized(game_init_progress: Query<&GameInitProgress>) -> bool
{
    Readiness::new(**game_init_progress.single()).is_ready()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_game_over_report(expected_scores: &HashMap<ClientId, PlayerScore>, game_over_report: GameOverReport)
{
    let game_over_report: ClickGameOverReport = game_over_report.get().unwrap();
    assert_eq!(expected_scores.len(), game_over_report.player_reports.len());

    for ClickPlayerReport{ client_id, score } in game_over_report.player_reports.iter()
    {
        let expected_score = expected_scores.get(&client_id).unwrap();
        assert_eq!(*score, *expected_score);
    }
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
fn game_instance_launcher_demo()
{
    // game info
    let num_players = 2;
    let num_watchers = 1;

    // config
    let ticks_per_sec   = 100;
    let max_init_ticks  = 2000;
    let game_play_ticks = 10;

    // server setup config
    let server_setup_config = GameServerSetupConfig{
            protocol_id  : get_test_protocol_id(),
            expire_secs  : 10u64,
            timeout_secs : 5i32,
            server_ip    : Ipv6Addr::LOCALHOST.into(),
            proxy_ip     : None,
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

    // make game instance launcher
    let game_launcher = GameInstanceLauncher::new(GameInstanceLauncherLocal::new(game_factory));


    // make init data for the clients
    let mut client_init_data = Vec::<ClickClientInitDataForGame>::with_capacity(num_players + num_watchers);

    for i in 0..num_players
    {
        client_init_data.push(make_player_init_for_game(gen_rand128(), ClientId::new(i as u64)));
    }

    for i in num_players..(num_players + num_watchers)
    {
        client_init_data.push(make_watcher_init_for_game(gen_rand128(), ClientId::new(i as u64)));
    }


    // make new game instance
    let (report_sender, mut report_receiver) = new_io_channel::<GameInstanceReport>();
    let launch_pack = ClickLaunchPackData{ config: game_factory_config, clients: client_init_data};
    let launch_pack = GameLaunchPack::new(0u64, launch_pack);
    let mut game_instance = game_launcher.launch(launch_pack, report_sender);
    std::thread::sleep(Duration::from_millis(30));
    assert!(game_instance.is_running());

    // extract game start report
    let Some(GameInstanceReport::GameStart(_, mut game_start_report)) = report_receiver.try_recv()
    else { panic!("failed to start game"); };

    // get token meta
    let token_meta = game_start_report.metas.native.unwrap();

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


    // tick all the clients until initialized
    loop
    {
        // wait a bit so the server can update in the game instance
        std::thread::sleep(Duration::from_millis(15));
        assert!(game_instance.is_running());

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
    std::thread::sleep(Duration::from_millis(40));  //wait for server to finalize initialization
    assert!(game_instance.is_running());

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

    // record expected final scores
    let mut expected_scores: HashMap<ClientId, PlayerScore> = HashMap::default();
    for i in 0..num_players { expected_scores.insert(ClientId::new(i as u64), PlayerScore{ score: 1 }); }

    // tick until game over report received
    let game_over_report: Option<GameOverReport>;
    loop
    {
        // allow server to tick
        std::thread::sleep(Duration::from_millis(15));

        // check for game over
        if let Some(GameInstanceReport::GameOver(_, report)) = report_receiver.try_recv()
        {
            // save game over report
            game_over_report = Some(report);

            // note: don't test if game instance is running here (it may or may not be)
            break;
        }
        else { assert!(game_instance.is_running()); }

        // tick clients
        for client in client_apps.iter_mut()
            { client.update(); }
    }

    // one more client update to make sure we collect pending information
    for client in client_apps.iter_mut()
        { client.update(); }


    // pull out watcher client
    let mut watcher_client_app = client_apps.pop().unwrap();

    // check test conditions
    check_game_over_report(&expected_scores, game_over_report.unwrap());

    for player_client in client_apps.iter_mut()
    {
        check_player_scores(player_client, num_players as u32, 1);
    }

    //todo: use information hiding to prevent watcher client from seeing scores (need bevy_replicon update)
    check_player_scores(&mut watcher_client_app, num_players as u32, 1);
}

//-------------------------------------------------------------------------------------------------------------------
