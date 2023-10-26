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

fn make_configs(ongoing_game_expiry: Duration) -> HostServerStartupPack
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
            expiry_duration: ongoing_game_expiry,
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
fn ongoing_game_expires()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let ongoing_game_expiry = Duration::from_millis(100);
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs(ongoing_game_expiry));

    // make a game hub client
    let (_, hub) = make_test_host_hub_client(host_hub_url);

    // make user clients
    let (user1_id, user1) = make_test_host_user_client(host_user_url.clone());
    let (user2_id, user2) = make_test_host_user_client(host_user_url);


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
    user2.request(UserToHostRequest::LobbySearch(LobbySearchRequest::Page{ youngest_lobby_id: u64::MAX, num_lobbies: 1 }))
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

    // - user 2 receives lobby data
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    // - users 1, 2 receive lobby state
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 launches lobby
    user1.send(UserToHostMsg::LaunchLobbyGame{ id: lobby.id }).expect("send failed");
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


    // users 1, 2 send acks
    user1.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    user2.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives game request
    let Some(HostHubServerVal::Msg(HostToHubMsg::StartGame(request))) = hub.next_val()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id);


    // game hub sends game start
    hub.send(HubToHostMsg::GameStart{ id: made_lobby_id, request, report: dummy_game_start_report(vec![user1_id, user2_id]) })
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive game start
    let Some(HostUserServerVal::Msg(HostToUserMsg::GameStart{ id: _, connect: _ })) = user1.next_val()
    else { panic!("client did not receive server msg"); };

    let Some(HostUserServerVal::Msg(HostToUserMsg::GameStart{ id: _, connect: _ })) = user2.next_val()
    else { panic!("client did not receive server msg"); };


    // wait for ongoing game to expire
    std::thread::sleep(ongoing_game_expiry + Duration::from_millis(15));


    // update host server to purge expired ongoing games
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives abort game
    let Some(HostHubServerVal::Msg(HostToHubMsg::AbortGame{ id: made_lobby_id })) = hub.next_val()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - users 1, 2 get game aborted
    let Some(HostUserServerVal::Msg(HostToUserMsg::GameAborted{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::GameAborted{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);


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
fn game_hub_disconnects()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let ongoing_game_expiry = Duration::from_secs(100);
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs(ongoing_game_expiry));

    // make a game hub client
    let (_, hub) = make_test_host_hub_client(host_hub_url);

    // make user clients
    let (user1_id, user1) = make_test_host_user_client(host_user_url.clone());
    let (user2_id, user2) = make_test_host_user_client(host_user_url);


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
    user2.request(UserToHostRequest::LobbySearch(LobbySearchRequest::Page{ youngest_lobby_id: u64::MAX, num_lobbies: 1 }))
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

    // - user 2 receives lobby data
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    // - users 1, 2 receive lobby state
    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::LobbyState{ lobby })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(lobby.id, made_lobby_id);


    // user 1 launches lobby
    user1.send(UserToHostMsg::LaunchLobbyGame{ id: lobby.id }).expect("send failed");
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


    // users 1, 2 send acks
    user1.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    user2.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub receives game request
    let Some(HostHubServerVal::Msg(HostToHubMsg::StartGame(request))) = hub.next_val()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id);


    // game hub sends game start
    hub.send(HubToHostMsg::GameStart{ id: made_lobby_id, request, report: dummy_game_start_report(vec![user1_id, user2_id]) })
        .expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive game start
    let Some(HostUserServerVal::Msg(HostToUserMsg::GameStart{ id: _, connect: _ })) = user1.next_val()
    else { panic!("client did not receive server msg"); };

    let Some(HostUserServerVal::Msg(HostToUserMsg::GameStart{ id: _, connect: _ })) = user2.next_val()
    else { panic!("client did not receive server msg"); };


    // disconnect the game hub
    hub.close();
    std::thread::sleep(Duration::from_millis(15));


    // update host server
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 get game aborted
    let Some(HostUserServerVal::Msg(HostToUserMsg::GameAborted{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    let Some(HostUserServerVal::Msg(HostToUserMsg::GameAborted{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);


    // - users 1, 2 receive nothing
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
