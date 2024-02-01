//local shortcuts
use crate::host_server::*;
use crate::test_helpers::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_fw::*;
use bevy_girk_host_server::*;

//third-party shortcuts

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn make_configs() -> HostServerStartupPack
{
    // configs
    let host_server_config = HostServerConfig{
            ticks_per_sec                   : None,  //we will manually update the host server
            ongoing_game_purge_period_ticks : 1u64,
        };
    let lobbies_cache_config = LobbiesCacheConfig{
            max_request_size: 10u16,
            lobby_checker: Box::new(BasicLobbyChecker{
                max_lobby_players     : 1u16,
                max_lobby_watchers    : 0u16,
                min_players_to_launch : 1u16,
            })
        };
    let pending_lobbies_cache_config = PendingLobbiesConfig{
            ack_timeout  : Duration::from_secs(10),
            start_buffer : Duration::from_secs(3),
        };
    let ongoing_games_cache_config = OngoingGamesCacheConfig{
            expiry_duration: Duration::from_secs(100),
        };
    let game_hub_disconnect_buffer_config = GameHubDisconnectBufferConfig{
            expiry_duration: Duration::from_secs(1),  //game hub stays in buffer after disconnecting
        };

    HostServerStartupPack{
            host_server_config,
            lobbies_cache_config,
            pending_lobbies_cache_config,
            ongoing_games_cache_config,
            game_hub_disconnect_buffer_config,
        }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn game_hub_reconnects()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs());

    // make a game hub client
    let (_, mut hub) = make_test_host_hub_client_with_id(0u128, host_hub_url.clone());

    // make user clients
    let (user1_id, mut user1) = make_test_host_user_client(host_user_url.clone());
    let (user2_id, mut user2) = make_test_host_user_client(host_user_url);

    // clients connected
    std::thread::sleep(Duration::from_millis(15));

    let HostHubClientEvent::Report(_) = hub.next().unwrap() else { unimplemented!(); };
    let HostUserClientEvent::Report(_) = user1.next().unwrap() else { unimplemented!(); };
    let HostUserClientEvent::Report(_) = user2.next().unwrap() else { unimplemented!(); };

    // hub initializes its capacity
    hub.send(HubToHostMsg::Capacity(GameHubCapacity(2)));
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));


    // user 1 makes lobby
    user1.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 recieves lobby
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user1.next()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id1 = lobby.id;


    // user 1 launches lobby
    user1.request(UserToHostRequest::LaunchLobbyGame{ id: made_lobby_id1 });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 receives ack request
    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id1);

    // - user 1 receives ack for launching the game
    let Some(HostUserClientEvent::Ack(_request_id)) = user1.next()
    else { panic!("client did not receive server msg"); };


    // user 1 sends ack
    user1.send(UserToHostMsg::AckPendingLobby{ id });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives game request
    let Some(HostHubClientEvent::Msg(HostToHubMsg::StartGame(request))) = hub.next()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id1);


    // game hub sends game start
    hub.send(HubToHostMsg::GameStart{ id: made_lobby_id1, request, report: dummy_game_start_report(vec![user1_id]) })
        ;
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 receives game start
    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameStart{ id: _, connect: _, start: _ })) = user1.next()
    else { panic!("client did not receive server msg"); };


    // disconnect the game hub
    hub.close();
    std::thread::sleep(Duration::from_millis(15));


    // update host server
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 get nothing (disconnected game hub is buffered)
    let None = user1.next() else { panic!("client received server msg unexpectedly"); };
    let None = user2.next() else { panic!("client received server msg unexpectedly"); };


    // user 2 makes lobby
    user2.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 recieves lobby
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user2.next()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id2 = lobby.id;


    // user 2 launches lobby
    user2.request(UserToHostRequest::LaunchLobbyGame{ id: made_lobby_id2 });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives ack request
    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id2);

    // - user 2 receives ack for launching the game
    let Some(HostUserClientEvent::Ack(_request_id)) = user2.next()
    else { panic!("client did not receive server msg"); };


    // user 2 sends ack
    user2.send(UserToHostMsg::AckPendingLobby{ id });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives ack fail (no game hub capacity)
    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id2);


    // reconnect the game hub
    let (_, mut hub) = make_test_host_hub_client_with_id(0u128, host_hub_url);

    // clients connected
    std::thread::sleep(Duration::from_millis(15));
    let HostHubClientEvent::Report(_) = hub.next().unwrap() else { unimplemented!(); };

    // hub initializes its capacity
    hub.send(HubToHostMsg::Capacity(GameHubCapacity(1)));
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));


    // user 2 launches lobby (try again now that hub is reconnected with non-zero capacity)
    user2.request(UserToHostRequest::LaunchLobbyGame{ id });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives ack request
    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id2);

    // - user 2 receives ack for launching the game
    let Some(HostUserClientEvent::Ack(_request_id)) = user2.next()
    else { panic!("client did not receive server msg"); };


    // user 2 sends ack
    user2.send(UserToHostMsg::AckPendingLobby{ id });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives game request
    let Some(HostHubClientEvent::Msg(HostToHubMsg::StartGame(request))) = hub.next()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id2);


    // game hub sends game start
    hub.send(HubToHostMsg::GameStart{ id: made_lobby_id2, request, report: dummy_game_start_report(vec![user2_id]) })
        ;
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives game start
    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameStart{ id: _, connect: _, start: _ })) = user2.next()
    else { panic!("client did not receive server msg"); };


    // hub sends game over reports
    hub.send(HubToHostMsg::GameOver{ id: made_lobby_id1, report: GameOverReport::default() });
    hub.send(HubToHostMsg::GameOver{ id: made_lobby_id2, report: GameOverReport::default() });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive game over reports for their games
    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameOver{ id, report: _ })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id1);

    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameOver{ id, report: _ })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id2);


    // - users 1, 2, and hub receive nothing
    let None = user1.next() else { panic!("client received server msg unexpectedly"); };
    let None = user2.next() else { panic!("client received server msg unexpectedly"); };
    let None = hub.next() else { panic!("hub received server msg unexpectedly"); };


    // cleanup
    let _ = user1.close();
    let _ = user2.close();
    let _ = hub.close();
    std::thread::sleep(Duration::from_millis(15));
}

//-------------------------------------------------------------------------------------------------------------------
