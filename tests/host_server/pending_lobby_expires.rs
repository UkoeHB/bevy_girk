//local shortcuts
use crate::host_server::*;
use crate::test_helpers::*;
use bevy_girk_backend_public::*;
use bevy_girk_host_server::*;

//third-party shortcuts

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn make_configs(ack_timeout: Duration, start_buffer: Duration) -> HostServerStartupPack
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
            ack_timeout  : ack_timeout,
            start_buffer : start_buffer,
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
fn pending_lobby_expires()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let ack_expiry   = Duration::from_millis(100);
    let start_buffer = ack_expiry + ack_expiry;

    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs(ack_expiry, start_buffer));

    // make a game hub client
    let (_, hub) = make_test_host_hub_client(host_hub_url);

    // make user clients
    let (_, user1) = make_test_host_user_client(host_user_url.clone());
    let (_, user2) = make_test_host_user_client(host_user_url);


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
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id = lobby.id;


    // user 2 accesses lobby info
    user2.request(UserToHostRequest::LobbySearch(LobbySearchRequest::PageOlder{ youngest_id: u64::MAX, num: 1 }))
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby response
    let Some(HostUserServerVal::Response(
            HostToUserResponse::LobbySearchResult(LobbySearchResult{ req: _, lobbies, num_younger: _, total: _ }), _
        )) = user2.next_val()
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

    // - user 2 receives lobby
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    // - users 1, 2 receive lobby update
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 launches lobby
    user1.request(UserToHostRequest::LaunchLobbyGame{ id: lobby.id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive ack requests
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 receives ack for launching the game
    let Some(HostUserServerVal::Ack(_request_id)) = user1.next_val()
    else { panic!("client did not receive server msg"); };


    // wait for ack expiry
    std::thread::sleep(ack_expiry);


    // update host server
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 get ack fails
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);


    // user 1 launches lobby again
    user1.request(UserToHostRequest::LaunchLobbyGame{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive ack requests
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 receives ack for launching the game
    let Some(HostUserServerVal::Ack(_request_id)) = user1.next_val()
    else { panic!("client did not receive server msg"); };


    // users 1, 2 ack the lobby
    user1.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    user2.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives game request
    let Some(HostHubServerVal::Msg(HostToHubMsg::StartGame(request))) = hub.next_val()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id);


    // wait for start buffer expiry
    std::thread::sleep(ack_expiry + start_buffer);


    // update host server
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 get ack fails
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - game hub receives nothing (host server only aborts the game hub's game when receiving a game start that can't be used)
    let None = hub.next_val() else { panic!("hub received server msg unexpectedly"); };


    // - hub, and users 1, 2 receive nothing
    let None = user1.next_val() else { panic!("client received server msg unexpectedly"); };
    let None = user2.next_val() else { panic!("client received server msg unexpectedly"); };
    let None = hub.next_val() else { panic!("hub received server msg unexpectedly"); };


    // cleanup
    let _ = user1.close();
    let _ = user2.close();
    let _ = hub.close();
    std::thread::sleep(Duration::from_millis(15));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn pending_lobby_expires_then_reacks()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let ack_expiry   = Duration::from_millis(100);
    let start_buffer = ack_expiry + ack_expiry;

    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs(ack_expiry, start_buffer));

    // make a game hub client
    let (_, hub) = make_test_host_hub_client(host_hub_url);

    // make user clients
    let (user1_id, user1) = make_test_host_user_client(host_user_url.clone());
    let (user2_id, user2) = make_test_host_user_client(host_user_url.clone());
    let (user3_id, user3) = make_test_host_user_client(host_user_url);


    // hub initializes its capacity
    hub.send(HubToHostMsg::Capacity(GameHubCapacity(10))).expect("send failed");
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
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id = lobby.id;


    // user 2 accesses lobby info
    user2.request(UserToHostRequest::LobbySearch(LobbySearchRequest::PageOlder{ youngest_id: u64::MAX, num: 1 }))
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby response
    let Some(HostUserServerVal::Response(
            HostToUserResponse::LobbySearchResult(LobbySearchResult{ req: _, lobbies, num_younger: _, total: _ }), _
        )) = user2.next_val()
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

    // - user 2 receives lobby
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    // - users 1, 2 receive lobby update
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 launches lobby
    user1.request(UserToHostRequest::LaunchLobbyGame{ id: lobby.id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive ack requests
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 receives ack for launching the game
    let Some(HostUserServerVal::Ack(_request_id)) = user1.next_val()
    else { panic!("client did not receive server msg"); };


    // users 1, 2 ack the lobby
    user1.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    user2.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives game request
    let Some(HostHubServerVal::Msg(HostToHubMsg::StartGame(request))) = hub.next_val()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id);


    // wait for start buffer expiry
    std::thread::sleep(ack_expiry + start_buffer);


    // update host server
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 get ack fails
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - game hub receives nothing (host server only aborts the game hub's game when receiving a game start that can't be used)
    let None = hub.next_val() else { panic!("hub received server msg unexpectedly"); };


    // user 2 leaves lobby
    user2.request(UserToHostRequest::LeaveLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby leave
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyLeave{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 receives lobby update
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);
    assert_eq!(lobby.members.len(), 1);

    // - user 2 receives ack for leaving the lobby
    let Some(HostUserServerVal::Ack(_request_id)) = user2.next_val()
    else { panic!("client did not receive server msg"); };


    // user 3 joins lobby
    user3.request(UserToHostRequest::JoinLobby{
            id     : made_lobby_id,
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test")
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 3 receives lobby
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user3.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    // - users 1, 3 receive lobby update
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user3.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 launches lobby
    user1.request(UserToHostRequest::LaunchLobbyGame{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 3 receive ack requests
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user3.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 receives ack for launching the game
    let Some(HostUserServerVal::Ack(_request_id)) = user1.next_val()
    else { panic!("client did not receive server msg"); };


    // users 1, 3 ack the lobby
    user1.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    user3.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives nothing (it already has a game start registered for this id)
    let None = hub.next_val() else { panic!("hub received server msg unexpectedly"); };


    // game hub sends game start for first game-start request (with users 1, 2)
    hub.send(HubToHostMsg::GameStart{ id: made_lobby_id, request, report: dummy_game_start_report(vec![user1_id, user2_id]) })
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives abort game (due to new lobby membership, the game start was rejected by host server)
    let Some(HostHubServerVal::Msg(HostToHubMsg::AbortGame{ id: made_lobby_id })) = hub.next_val()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - game hub receives nothing else
    let None = hub.next_val() else { panic!("hub received server msg unexpectedly"); };


    // game hub sends reject game for aborted game
    hub.send(HubToHostMsg::AbortGame{ id: made_lobby_id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives game request (with new lobby membership)
    let Some(HostHubServerVal::Msg(HostToHubMsg::StartGame(request))) = hub.next_val()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id);


    // game hub sends game start for second game-start request (with users 1, 3)
    hub.send(HubToHostMsg::GameStart{ id: made_lobby_id, request, report: dummy_game_start_report(vec![user1_id, user3_id]) })
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 3 receive game start
    let Some(HostUserServerVal::Msg(HostToUserMsg::GameStart{ id: _, connect: _ })) = user1.next_val()
    else { panic!("client did not receive server msg"); };

    let Some(HostUserServerVal::Msg(HostToUserMsg::GameStart{ id: _, connect: _ })) = user3.next_val()
    else { panic!("client did not receive server msg"); };


    // - hub, and users 1, 2, 3 receive nothing
    let None = user1.next_val() else { panic!("client received server msg unexpectedly"); };
    let None = user2.next_val() else { panic!("client received server msg unexpectedly"); };
    let None = user3.next_val() else { panic!("client received server msg unexpectedly"); };
    let None = hub.next_val() else { panic!("hub received server msg unexpectedly"); };


    // cleanup
    let _ = user1.close();
    let _ = user2.close();
    let _ = user3.close();
    let _ = hub.close();
    std::thread::sleep(Duration::from_millis(15));
}

//-------------------------------------------------------------------------------------------------------------------
