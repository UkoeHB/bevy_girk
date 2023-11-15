//local shortcuts
use crate::host_server::*;
use crate::test_helpers::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_fw::*;
use bevy_girk_host_server::*;

//third-party shortcuts
use bevy_simplenet::ClientReport;

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
            max_request_size      : 10u16,
            lobby_checker: Box::new(BasicLobbyChecker{
                max_lobby_players     : 2u16,
                max_lobby_watchers    : 0u16,
                min_players_to_launch : 2u16,
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
            expiry_duration: Duration::from_secs(0),
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
fn client_reconnects_to_lobby()
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
    let (_, hub) = make_test_host_hub_client(host_hub_url);

    // make user clients
    let (_, user1) = make_test_host_user_client(host_user_url.clone());
    let (user2_id, user2) = make_test_host_user_client(host_user_url.clone());

    // clients connected
    std::thread::sleep(Duration::from_millis(15));

    let HostHubClientEvent::Report(_) = hub.next().unwrap() else { unimplemented!(); };
    let HostUserClientEvent::Report(_) = user1.next().unwrap() else { unimplemented!(); };
    let HostUserClientEvent::Report(_) = user2.next().unwrap() else { unimplemented!(); };

    // hub initializes its capacity
    hub.send(HubToHostMsg::Capacity(GameHubCapacity(1))).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));


    // user 1 makes lobby
    user1.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 recieves lobby
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user1.next()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id = lobby.id;


    // user 2 accesses lobby info
    user2.request(UserToHostRequest::LobbySearch(LobbySearchRequest::PageOlder{ youngest_id: u64::MAX, num: 1 }))
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby response
    let Some(HostUserClientEvent::Response(
            HostToUserResponse::LobbySearchResult(LobbySearchResult{ req: _, lobbies, num_younger: _, total: _ }), _
        )) = user2.next()
    else { panic!("client did not receive server msg"); };

    let lobby = lobbies.get(0).expect("there should be one lobby");
    assert_eq!(lobby.id, made_lobby_id);


    // user 2 joins lobby
    user2.request(UserToHostRequest::JoinLobby{
            id     : made_lobby_id,
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test")
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby data
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    // - users 1, 2 receive lobby state
    let Some(HostUserClientEvent::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserClientEvent::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 2 disconnects from host server
    user2.close();
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 receives lobby update
    let Some(HostUserClientEvent::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 2 reconnects to host server
    let (_, user2) = make_test_host_user_client_with_id(user2_id, host_user_url);
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 connected
    let HostUserClientEvent::Report(ClientReport::Connected) = user2.next().unwrap() else { unimplemented!(); };

    // - hub, and users 1, 2 receive nothing
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

#[test]
fn client_reconnects_to_pending_lobby()
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
    let (_, hub) = make_test_host_hub_client(host_hub_url);

    // make user clients
    let (_, user1) = make_test_host_user_client(host_user_url.clone());
    let (user2_id, user2) = make_test_host_user_client(host_user_url.clone());

    // clients connected
    std::thread::sleep(Duration::from_millis(15));

    let HostHubClientEvent::Report(_) = hub.next().unwrap() else { unimplemented!(); };
    let HostUserClientEvent::Report(_) = user1.next().unwrap() else { unimplemented!(); };
    let HostUserClientEvent::Report(_) = user2.next().unwrap() else { unimplemented!(); };

    // hub initializes its capacity
    hub.send(HubToHostMsg::Capacity(GameHubCapacity(1))).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));


    // user 1 makes lobby
    user1.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 recieves lobby
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user1.next()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id = lobby.id;


    // user 2 accesses lobby info
    user2.request(UserToHostRequest::LobbySearch(LobbySearchRequest::PageOlder{ youngest_id: u64::MAX, num: 1 }))
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby response
    let Some(HostUserClientEvent::Response(
            HostToUserResponse::LobbySearchResult(LobbySearchResult{ req: _, lobbies, num_younger: _, total: _ }), _
        )) = user2.next()
    else { panic!("client did not receive server msg"); };

    let lobby = lobbies.get(0).expect("there should be one lobby");
    assert_eq!(lobby.id, made_lobby_id);


    // user 2 joins lobby
    user2.request(UserToHostRequest::JoinLobby{
            id     : made_lobby_id,
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test")
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby data
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    // - users 1, 2 receive lobby state
    let Some(HostUserClientEvent::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserClientEvent::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 launches lobby
    user1.request(UserToHostRequest::LaunchLobbyGame{ id: lobby.id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive ack requests
    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 receives ack for launching the game
    let Some(HostUserClientEvent::Ack(_request_id)) = user1.next()
    else { panic!("client did not receive server msg"); };


    // user 2 disconnects from host server (this nacks the lobby)
    user2.close();
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 receives ack fail
    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 receives lobby update
    let Some(HostUserClientEvent::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    // - game hub receives nothing (game did not launch)
    let None = hub.next() else { panic!("hub received server msg unexpectedly"); };


    // user 2 reconnects to host server
    let (_, user2) = make_test_host_user_client_with_id(user2_id, host_user_url);
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 connected
    let HostUserClientEvent::Report(ClientReport::Connected) = user2.next().unwrap() else { unimplemented!(); };

    // - users 1, 2 receive nothing
    let None = user1.next() else { panic!("client received server msg unexpectedly"); };
    let None = user2.next() else { panic!("client received server msg unexpectedly"); };


    // cleanup
    let _ = user1.close();
    let _ = user2.close();
    let _ = hub.close();
    std::thread::sleep(Duration::from_millis(15));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn client_leaves_full_acked_pending_lobby()
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
    let (_, hub) = make_test_host_hub_client(host_hub_url);

    // make user clients
    let (user1_id, user1) = make_test_host_user_client(host_user_url.clone());
    let (user2_id, user2) = make_test_host_user_client(host_user_url.clone());

    // clients connected
    std::thread::sleep(Duration::from_millis(15));

    let HostHubClientEvent::Report(_) = hub.next().unwrap() else { unimplemented!(); };
    let HostUserClientEvent::Report(_) = user1.next().unwrap() else { unimplemented!(); };
    let HostUserClientEvent::Report(_) = user2.next().unwrap() else { unimplemented!(); };

    // hub initializes its capacity
    hub.send(HubToHostMsg::Capacity(GameHubCapacity(1))).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));


    // user 1 makes lobby
    user1.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 recieves lobby
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user1.next()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id = lobby.id;


    // user 2 accesses lobby info
    user2.request(UserToHostRequest::LobbySearch(LobbySearchRequest::PageOlder{ youngest_id: u64::MAX, num: 1 }))
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby response
    let Some(HostUserClientEvent::Response(
            HostToUserResponse::LobbySearchResult(LobbySearchResult{ req: _, lobbies, num_younger: _, total: _ }), _
        )) = user2.next()
    else { panic!("client did not receive server msg"); };

    let lobby = lobbies.get(0).expect("there should be one lobby");
    assert_eq!(lobby.id, made_lobby_id);


    // user 2 joins lobby
    user2.request(UserToHostRequest::JoinLobby{
            id     : made_lobby_id,
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test")
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby data
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    // - users 1, 2 receive lobby state
    let Some(HostUserClientEvent::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserClientEvent::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 launches lobby
    user1.request(UserToHostRequest::LaunchLobbyGame{ id: lobby.id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive ack requests
    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 receives ack for launching the game
    let Some(HostUserClientEvent::Ack(_request_id)) = user1.next()
    else { panic!("client did not receive server msg"); };


    // users 1, 2 send acks
    user1.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    user2.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives game request
    let Some(HostHubClientEvent::Msg(HostToHubMsg::StartGame(request))) = hub.next()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id);


    // user 1 disconnects from host server (this nacks the lobby and closes the lobby)
    user1.close();
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives ack fail
    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 2 receives leave lobby
    let Some(HostUserClientEvent::Msg(HostToUserMsg::LobbyLeave{ id })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);


    // game hub sends start game
    hub.send(HubToHostMsg::GameStart{ id: made_lobby_id, request, report: dummy_game_start_report(vec![user1_id, user2_id]) })
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives abort game
    let Some(HostHubClientEvent::Msg(HostToHubMsg::AbortGame{ id: made_lobby_id })) = hub.next()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(id, made_lobby_id);


    // user 1 reconnects to host server
    let (_, user1) = make_test_host_user_client_with_id(user1_id, host_user_url);
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 connected
    let HostUserClientEvent::Report(ClientReport::Connected) = user1.next().unwrap() else { unimplemented!(); };


    // - hub, and users 1, 2 receive nothing
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

#[test]
fn client_reconnects_to_game()
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
    let (_, hub) = make_test_host_hub_client(host_hub_url);

    // make user clients
    let (user1_id, user1) = make_test_host_user_client(host_user_url.clone());
    let (user2_id, user2) = make_test_host_user_client(host_user_url.clone());

    // clients connected
    std::thread::sleep(Duration::from_millis(15));

    let HostHubClientEvent::Report(_) = hub.next().unwrap() else { unimplemented!(); };
    let HostUserClientEvent::Report(_) = user1.next().unwrap() else { unimplemented!(); };
    let HostUserClientEvent::Report(_) = user2.next().unwrap() else { unimplemented!(); };

    // hub initializes its capacity
    hub.send(HubToHostMsg::Capacity(GameHubCapacity(1))).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));


    // user 1 makes lobby
    user1.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 recieves lobby
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user1.next()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id = lobby.id;


    // user 2 accesses lobby info
    user2.request(UserToHostRequest::LobbySearch(LobbySearchRequest::PageOlder{ youngest_id: u64::MAX, num: 1 }))
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby response
    let Some(HostUserClientEvent::Response(
            HostToUserResponse::LobbySearchResult(LobbySearchResult{ req: _, lobbies, num_younger: _, total: _ }), _
        )) = user2.next()
    else { panic!("client did not receive server msg"); };

    let lobby = lobbies.get(0).expect("there should be one lobby");
    assert_eq!(lobby.id, made_lobby_id);


    // user 2 joins lobby
    user2.request(UserToHostRequest::JoinLobby{
            id     : made_lobby_id,
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test")
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby data
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    // - users 1, 2 receive lobby state
    let Some(HostUserClientEvent::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserClientEvent::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 launches lobby
    user1.request(UserToHostRequest::LaunchLobbyGame{ id: lobby.id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive ack requests
    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 receives ack for launching the game
    let Some(HostUserClientEvent::Ack(_request_id)) = user1.next()
    else { panic!("client did not receive server msg"); };


    // users 1, 2 send acks
    user1.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    user2.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives game request
    let Some(HostHubClientEvent::Msg(HostToHubMsg::StartGame(request))) = hub.next()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id);


    // game hub sends game start
    hub.send(HubToHostMsg::GameStart{ id: made_lobby_id, request, report: dummy_game_start_report(vec![user1_id, user2_id]) })
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive game start
    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameStart{ id: _, connect: _ })) = user1.next()
    else { panic!("client did not receive server msg"); };

    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameStart{ id: _, connect: _ })) = user2.next()
    else { panic!("client did not receive server msg"); };


    // user 1 disconnects from host server
    user1.close();
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));


    // user 1 reconnects to host server
    let (_, user1) = make_test_host_user_client_with_id(user1_id, host_user_url);
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 connected
    let HostUserClientEvent::Report(ClientReport::Connected) = user1.next().unwrap() else { unimplemented!(); };

    // - user 1 receives game start
    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameStart{ id: _, connect: _ })) = user1.next()
    else { panic!("client did not receive server msg"); };


    // game hub sends game over
    hub.send(HubToHostMsg::GameOver{ id: made_lobby_id, report: GameOverReport::default() }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive game over report
    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameOver{ id, report: _ })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameOver{ id, report: _ })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);


    // - hub, and users 1, 2 receive nothing
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
