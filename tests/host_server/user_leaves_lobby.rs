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

fn make_configs(lobby_size: u16) -> HostServerStartupPack
{
    // configs
    let host_server_config = HostServerConfig{
            ticks_per_sec                   : None,  //we will manually update the host server
            ongoing_game_purge_period_ticks : 1u64,
        };
    let lobbies_cache_config = LobbiesCacheConfig{
            max_request_size      : 10u16,
            lobby_checker: Box::new(BasicLobbyChecker{
                max_lobby_players     : lobby_size,
                max_lobby_watchers    : 0u16,
                min_players_to_launch : lobby_size,
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
fn client_leaves_lobby()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs(3u16));

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
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ id: made_lobby_id, lobby: _ }, _)) = user1.next_val()
    else { panic!("client did not receive server msg"); };


    // user 2 accesses lobby info
    user2.request(UserToHostRequest::GetLobby(LobbySearchType::Page{ youngest_lobby_id: u64::MAX, num_lobbies: 1 }))
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby response
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbySearchResult{ request: _, lobbies }, _)) = user2.next_val()
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
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ id, lobby: _}, _)) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - users 1, 2 receive lobby update
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 2 leaves lobby
    user2.send(UserToHostMsg::LeaveLobby{ id }).expect("send failed");
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
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ id, lobby: _}, _)) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - users 1, 2 receive lobby update
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 leaves lobby (user 1 is the owner so the lobby should be removed)
    user1.send(UserToHostMsg::LeaveLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive lobby leave
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyLeave{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyLeave{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);


    // user 1 requests lobby list
    user1.request(UserToHostRequest::GetLobby(LobbySearchType::Page{ youngest_lobby_id: u64::MAX, num_lobbies: 1 }))
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 receives lobby response
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbySearchResult{ request: _, lobbies }, _)) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobbies.len(), 0);


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
fn client_leaves_pending_lobby()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs(2u16));

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
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ id: made_lobby_id, lobby: _ }, _)) = user1.next_val()
    else { panic!("client did not receive server msg"); };


    // user 2 accesses lobby info
    user2.request(UserToHostRequest::GetLobby(LobbySearchType::Page{ youngest_lobby_id: u64::MAX, num_lobbies: 1 }))
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby response
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbySearchResult{ request: _, lobbies }, _)) = user2.next_val()
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
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ id, lobby: _}, _)) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - users 1, 2 receive lobby update
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 launches lobby
    user1.send(UserToHostMsg::LaunchLobbyGame{ id }).expect("send failed");
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


    // user 2 leaves lobby
    user2.send(UserToHostMsg::LeaveLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 get ack fails
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 2 receives lobby leave
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyLeave{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 receives lobby update
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);
    assert_eq!(lobby.members.len(), 1);


    // user 2 joins lobby again
    user2.request(UserToHostRequest::JoinLobby{
            id     : made_lobby_id,
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test")
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ id, lobby: _}, _)) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - users 1, 2 receive lobby update
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 launches the lobby again
    user1.send(UserToHostMsg::LaunchLobbyGame{ id }).expect("send failed");
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


    // user 2 leaves the lobby
    user2.send(UserToHostMsg::LeaveLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive nothing (leave lobby failed because pending lobby is fully acked)
    let None = user1.next_val() else { panic!("client received server msg unexpectedly"); };
    let None = user2.next_val() else { panic!("client received server msg unexpectedly"); };


    // - hub receives nothing
    let None = hub.next_val() else { panic!("hub received server msg unexpectedly"); };


    // cleanup
    let _ = user1.close();
    let _ = user2.close();
    let _ = hub.close();
    std::thread::sleep(Duration::from_millis(15));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn client_resets_pending_lobby()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs(2u16));

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
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ id: made_lobby_id, lobby: _ }, _)) = user1.next_val()
    else { panic!("client did not receive server msg"); };


    // user 2 accesses lobby info
    user2.request(UserToHostRequest::GetLobby(LobbySearchType::Page{ youngest_lobby_id: u64::MAX, num_lobbies: 1 }))
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives lobby response
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbySearchResult{ request: _, lobbies }, _)) = user2.next_val()
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
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ id, lobby: _}, _)) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - users 1, 2 receive lobby update
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 launches lobby
    user1.send(UserToHostMsg::LaunchLobbyGame{ id }).expect("send failed");
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


    // user 1 resets his state
    let reset_request = user1.request(UserToHostRequest::ResetLobby).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 get ack fails
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - users, 1, 2 receive lobby leave
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyLeave{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyLeave{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 gets request ack (the user is now synchronized with the server's output stream)
    let Some(HostUserServerVal::Ack(ack_id)) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(reset_request.id(), ack_id);


    // - users 1, 2 receive nothing (leave lobby failed because pending lobby is fully acked)
    let None = user1.next_val() else { panic!("client received server msg unexpectedly"); };
    let None = user2.next_val() else { panic!("client received server msg unexpectedly"); };


    // - hub receives nothing
    let None = hub.next_val() else { panic!("hub received server msg unexpectedly"); };


    // cleanup
    let _ = user1.close();
    let _ = user2.close();
    let _ = hub.close();
    std::thread::sleep(Duration::from_millis(15));
}

//-------------------------------------------------------------------------------------------------------------------
