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

fn make_configs() -> GameHubServerStartupPack
{
    let game_hub_server_config = GameHubServerConfig{
            ticks_per_sec                   : None,
            initial_max_capacity            : 10u16,
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
fn game_lifecycle()
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
            make_configs(),
            100,
            3,
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
    assert!(initial_capacity > GameHubCapacity(0));


    // request game start
    let game_id = 0u64;
    let mut members = std::collections::HashMap::<u128, LobbyMemberData>::default();
    members.insert(0u128, LobbyMemberData{ env: bevy_simplenet::env_type(), color: LobbyMemberColor(0u64)});
    let start_request = GameStartRequest{ lobby_data: LobbyData{ id: game_id, members, ..Default::default() } };
    host_hub_server.send(connected_hub_id, HostToHubMsg::StartGame(start_request)).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - updated capacity
    // note: updating capacity races with the game start message
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Capacity(capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(capacity, GameHubCapacity(initial_capacity.0 - 1));


    // another update to collect and forward the game start report
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game start report
    let Some((hub_id, HostHubServerEvent::Msg(HubToHostMsg::GameStart{ id, request: _, report: _ }))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(hub_id, connected_hub_id);
    assert_eq!(id, game_id);



    // update again to collect game over report
    std::thread::sleep(Duration::from_millis(45));
    hub_server_app.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game over report
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::GameOver{ id, report: _ }))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(id, game_id);

    // - updated capacity
    let Some((_, HostHubServerEvent::Msg(HubToHostMsg::Capacity(capacity)))) = host_hub_server.next()
    else { panic!("host hub server did not receive game hub server msg"); };
    assert_eq!(capacity, initial_capacity);


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
