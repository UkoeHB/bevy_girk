//local shortcuts
use crate::game_hub_server::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_hub_server::*;
use bevy_girk_host_server::*;

//third-party shortcuts

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn make_configs(initial_max_capacity: u16) -> GameHubServerStartupPack
{
    let game_hub_server_config = GameHubServerConfig{
            ticks_per_sec                   : None,
            initial_max_capacity            : initial_max_capacity,
            running_game_purge_period_ticks : 100u64,
        };
    let pending_games_cache_config = PendingGamesCacheConfig{
            expiry_duration: Duration::from_secs(2),
        };
    let running_games_cache_config = RunningGamesCacheConfig{
            expiry_duration: Duration::from_secs(20),
        };

    GameHubServerStartupPack{
            game_hub_server_config,
            pending_games_cache_config,
            running_games_cache_config,
        }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn reject_duplicate_game()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a websocket host server
    let host_hub_server = make_test_host_hub_server();

    // make a game hub server
    let (hub_command_sender, mut hub_server_app) = make_test_game_hub_server(
            host_hub_server.url(),
            false,
            make_configs(10u16),
            100,
            20,
            Some(true)
        );
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - hub connects to server
    let Some((connected_hub_id, HostHubServerEvent::Report(HostHubServerReport::Connected(_, _)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server connection report"); };


    // update to get hub initial capacity now that we are connected
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - receive initial capacity
    let Some((hub_id, HostHubServerEvent::Msg(HubToHostMsg::Capacity(initial_capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(hub_id, connected_hub_id);
    assert_eq!(initial_capacity, GameHubCapacity(10));


    // request game start
    let game_id_1 = 1u64;
    let start_request_1 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_1, ..Default::default() } };
    host_hub_server.send(connected_hub_id, HostToHubMsg::StartGame(start_request_1)).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - updated capacity
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Capacity(capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(capacity, GameHubCapacity(initial_capacity.0 - 1));


    // another update to collect and forward the game start report
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game start report
    let Some((
            hub_id,
            HostHubServerEvent::Msg(HubToHostMsg::GameStart{ id, request: _, report: _ })
        )) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(hub_id, connected_hub_id);
    assert_eq!(id, game_id_1);


    // request game start for game 1 again
    let start_request_1 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_1, ..Default::default() } };
    host_hub_server.send(connected_hub_id, HostToHubMsg::StartGame(start_request_1)).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game ignored (duplicate)
    let None = host_hub_server.next() else { panic!("received msg unexpectedly"); };


    // request game start for game 2
    let game_id_2 = 2u64;
    let start_request_2 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_2, ..Default::default() } };
    host_hub_server.send(connected_hub_id, HostToHubMsg::StartGame(start_request_2)).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - updated capacity
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Capacity(capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(capacity, GameHubCapacity(initial_capacity.0 - 2));


    // another update to collect and forward the game start report
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game start report
    let Some((
            hub_id,
            HostHubServerEvent::Msg(HubToHostMsg::GameStart{ id, request: _, report: _ })
        )) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(hub_id, connected_hub_id);
    assert_eq!(id, game_id_2);


    // set hub max capacity to 0
    hub_command_sender.send(GameHubCommand::SetMaxCapacity(GameHubCapacity(0u16))).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - updated capacity
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Capacity(capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(capacity, GameHubCapacity(0u16));


    // shut down hub
    hub_command_sender.send(GameHubCommand::ShutDown).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game 1 or 2 aborted (shutting down)
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Abort{ id }))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert!((id == game_id_1) || (id == game_id_2));

    // - game 1 or 2 aborted (shutting down)
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Abort{ id }))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert!((id == game_id_1) || (id == game_id_2));

    // - hub disconnects from server
    let Some((disconnected_hub_id, HostHubServerEvent::Report(HostHubServerReport::Disconnected))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server connection report"); };
    assert_eq!(disconnected_hub_id, connected_hub_id);


    // - host hub server receives nothing else
    let None = host_hub_server.next() else { panic!("received msg unexpectedly"); };
    let None = host_hub_server.next() else { panic!("received connection report unexpectedly"); };
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn reject_no_capacity()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a websocket host server
    let host_hub_server = make_test_host_hub_server();

    // make a game hub server
    let (hub_command_sender, mut hub_server_app) = make_test_game_hub_server(
            host_hub_server.url(),
            false,
            make_configs(1u16),
            100,
            7,
            Some(true)
        );
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - hub connects to server
    let Some((connected_hub_id, HostHubServerEvent::Report(HostHubServerReport::Connected(_, _)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server connection report"); };


    // update to get hub initial capacity now that we are connected
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - receive initial capacity
    let Some((hub_id, HostHubServerEvent::Msg(HubToHostMsg::Capacity(initial_capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(hub_id, connected_hub_id);
    assert_eq!(initial_capacity, GameHubCapacity(1));


    // request game start
    let game_id_1 = 1u64;
    let start_request_1 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_1, ..Default::default() } };
    host_hub_server.send(connected_hub_id, HostToHubMsg::StartGame(start_request_1)).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - updated capacity
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Capacity(capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(capacity, GameHubCapacity(0));


    // another update to collect and forward the game start report
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game start report
    let Some((
            hub_id,
            HostHubServerEvent::Msg(HubToHostMsg::GameStart{ id, request: _, report: _ })
        )) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(hub_id, connected_hub_id);
    assert_eq!(id, game_id_1);


    // request game start for game 2
    let game_id_2 = 2u64;
    let start_request_2 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_2, ..Default::default() } };
    host_hub_server.send(connected_hub_id, HostToHubMsg::StartGame(start_request_2)).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game aborted (no capacity)
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Abort{ id }))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(id, game_id_2);


    // wait for first game to finish
    std::thread::sleep(Duration::from_millis(50));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game over report
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::GameOver{ id, report: _ }))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(id, game_id_1);

    // - updated capacity
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Capacity(capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(capacity, initial_capacity);


    // request game start for game 2
    let game_id_2 = 2u64;
    let start_request_2 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_2, ..Default::default() } };
    host_hub_server.send(connected_hub_id, HostToHubMsg::StartGame(start_request_2)).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - updated capacity
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Capacity(capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(capacity, GameHubCapacity(0));


    // another update to collect and forward the game start report
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game start report
    let Some((
            hub_id,
            HostHubServerEvent::Msg(HubToHostMsg::GameStart{ id, request: _, report: _ })
        )) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(hub_id, connected_hub_id);
    assert_eq!(id, game_id_2);


    // set hub max capacity to 0
    hub_command_sender.send(GameHubCommand::SetMaxCapacity(GameHubCapacity(0u16))).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - don't receive updated capacity (capacity is already zero, so changing max capacity doesn't affect it)
    let None = host_hub_server.next() else { panic!("received msg unexpectedly"); };


    // shut down hub
    hub_command_sender.send(GameHubCommand::ShutDown).unwrap();
    std::thread::sleep(Duration::from_millis(30));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(30));

    // - game 2 either finishes or gets aborted (shutting down)
    let Some(_) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };

    // - hub disconnects from server
    let Some((disconnected_hub_id, HostHubServerEvent::Report(HostHubServerReport::Disconnected))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server connection report"); };
    assert_eq!(disconnected_hub_id, connected_hub_id);


    // - host hub server receives nothing else
    let None = host_hub_server.next() else { panic!("received msg unexpectedly"); };
    let None = host_hub_server.next() else { panic!("received connection report unexpectedly"); };
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn reject_launch_pack_fail()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a websocket host server
    let host_hub_server = make_test_host_hub_server();

    // make a game hub server
    let (hub_command_sender, mut hub_server_app) = make_test_game_hub_server(
            host_hub_server.url(),
            false,
            make_configs(10u16),
            100,
            20,
            Some(false)  //cause launch pack source to fail
        );
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - hub connects to server
    let Some((connected_hub_id, HostHubServerEvent::Report(HostHubServerReport::Connected(_, _)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server connection report"); };


    // update to get hub initial capacity now that we are connected
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - receive initial capacity
    let Some((hub_id, HostHubServerEvent::Msg(HubToHostMsg::Capacity(initial_capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(hub_id, connected_hub_id);
    assert_eq!(initial_capacity, GameHubCapacity(10));


    // request game start
    let game_id_1 = 1u64;
    let start_request_1 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_1, ..Default::default() } };
    host_hub_server.send(connected_hub_id, HostToHubMsg::StartGame(start_request_1)).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game 1 aborted (launch pack failed)
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Abort{ id }))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(id, game_id_1);


    // set hub max capacity to 0
    hub_command_sender.send(GameHubCommand::SetMaxCapacity(GameHubCapacity(0u16))).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - updated capacity
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Capacity(capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(capacity, GameHubCapacity(0u16));


    // shut down hub
    hub_command_sender.send(GameHubCommand::ShutDown).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - hub disconnects from server
    let Some((disconnected_hub_id, HostHubServerEvent::Report(HostHubServerReport::Disconnected))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server connection report"); };
    assert_eq!(disconnected_hub_id, connected_hub_id);


    // - host hub server receives nothing else
    let None = host_hub_server.next() else { panic!("received msg unexpectedly"); };
    let None = host_hub_server.next() else { panic!("received connection report unexpectedly"); };
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn reject_hub_shutdown()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a websocket host server
    let host_hub_server = make_test_host_hub_server();

    // make a game hub server
    let (hub_command_sender, mut hub_server_app) = make_test_game_hub_server(
            host_hub_server.url(),
            false,
            make_configs(10u16),
            100,
            20,
            Some(true)
        );
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - hub connects to server
    let Some((connected_hub_id, HostHubServerEvent::Report(HostHubServerReport::Connected(_, _)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server connection report"); };


    // update to get hub initial capacity now that we are connected
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - receive initial capacity
    let Some((hub_id, HostHubServerEvent::Msg(HubToHostMsg::Capacity(initial_capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(hub_id, connected_hub_id);
    assert_eq!(initial_capacity, GameHubCapacity(10));


    // request game start
    let game_id_1 = 1u64;
    let start_request_1 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_1, ..Default::default() } };
    host_hub_server.send(connected_hub_id, HostToHubMsg::StartGame(start_request_1)).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - updated capacity
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Capacity(capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(capacity, GameHubCapacity(initial_capacity.0 - 1));


    // another update to collect and forward the game start report
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game start report
    let Some((
            hub_id,
            HostHubServerEvent::Msg(HubToHostMsg::GameStart{ id, request: _, report: _ })
        )) =host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(hub_id, connected_hub_id);
    assert_eq!(id, game_id_1);


    // set hub max capacity to 0
    hub_command_sender.send(GameHubCommand::SetMaxCapacity(GameHubCapacity(0u16))).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - updated capacity
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Capacity(capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(capacity, GameHubCapacity(0u16));


    // shut down hub
    hub_command_sender.send(GameHubCommand::ShutDown).unwrap();
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game 1 aborted (shutting down)
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Abort{ id }))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(id, game_id_1);

    // - hub disconnects from server
    let Some((disconnected_hub_id, HostHubServerEvent::Report(HostHubServerReport::Disconnected))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server connection report"); };
    assert_eq!(disconnected_hub_id, connected_hub_id);


    // - host hub server receives nothing else
    let None = host_hub_server.next() else { panic!("received msg unexpectedly"); };
    let None = host_hub_server.next() else { panic!("received connection report unexpectedly"); };
}

//-------------------------------------------------------------------------------------------------------------------
