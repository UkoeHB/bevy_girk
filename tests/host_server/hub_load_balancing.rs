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

// goal: if a hub is not responsive, then other hubs will pick up lobbies as the non-responsive hub accumulates pending
//       game requests
#[test]
fn host_load_balancing()
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

    // make game hub clients
    let (_, hub1) = make_test_host_hub_client_with_id(0u128, host_hub_url.clone());
    let (_, hub2) = make_test_host_hub_client_with_id(1u128, host_hub_url);  //with id to ensure sort order for hub selection

    // make user clients
    let (_, user1) = make_test_host_user_client(host_user_url.clone());
    let (_, user2) = make_test_host_user_client(host_user_url.clone());
    let (_, user3) = make_test_host_user_client(host_user_url.clone());
    let (_, user4) = make_test_host_user_client(host_user_url);


    // hubs initialize their capacity
    hub1.send(HubToHostMsg::Capacity(GameHubCapacity(1))).expect("send failed");  // 1 capacity
    hub2.send(HubToHostMsg::Capacity(GameHubCapacity(2))).expect("send failed");  // 2 capacity
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));


    // users make lobbies
    user1.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        }).expect("send failed");
    user2.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        }).expect("send failed");
    user3.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        }).expect("send failed");
    user4.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users recieve lobbies
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id1 = lobby.id;
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id2 = lobby.id;
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user3.next_val()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id3 = lobby.id;
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user4.next_val()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id4 = lobby.id;

    // users launch lobbies
    user1.send(UserToHostMsg::LaunchLobbyGame{ id: made_lobby_id1 }).expect("send failed");
    user2.send(UserToHostMsg::LaunchLobbyGame{ id: made_lobby_id2 }).expect("send failed");
    user3.send(UserToHostMsg::LaunchLobbyGame{ id: made_lobby_id3 }).expect("send failed");
    user4.send(UserToHostMsg::LaunchLobbyGame{ id: made_lobby_id4 }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - users receive ack requests
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user1.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id1);
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user2.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id2);
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user3.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id3);
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user4.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id4);


    // user 1 sends ack
    user1.send(UserToHostMsg::AckPendingLobby{ id: made_lobby_id1 }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub 2 receives game request
    let Some(HostHubServerVal::Msg(HostToHubMsg::StartGame(request))) = hub2.next_val()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id1);


    // user 2 sends ack
    user2.send(UserToHostMsg::AckPendingLobby{ id: made_lobby_id2 }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub 2 receives game request
    let Some(HostHubServerVal::Msg(HostToHubMsg::StartGame(request))) = hub2.next_val()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id2);


    // user 3 sends ack
    user3.send(UserToHostMsg::AckPendingLobby{ id: made_lobby_id3 }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - game hub 1 receives game request
    let Some(HostHubServerVal::Msg(HostToHubMsg::StartGame(request))) = hub1.next_val()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id3);


    // user 4 sends ack
    user4.send(UserToHostMsg::AckPendingLobby{ id: made_lobby_id4 }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 4 receives ack fail (no hubs with capacity)
    let Some(HostUserServerVal::Msg(HostToUserMsg::PendingLobbyAckFail{ id })) = user4.next_val()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id4);


    // hub 2 sends reject game
    hub2.send(HubToHostMsg::AbortGame{ id: made_lobby_id1 }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - hub 2 receives game request (aborted game tries to re-start on hub 2)
    let Some(HostHubServerVal::Msg(HostToHubMsg::StartGame(request))) = hub2.next_val()
    else { panic!("hub did not receive server msg"); };
    assert_eq!(request.game_id(), made_lobby_id1);


    // - hubs and users receive nothing
    let None = user1.next_val() else { panic!("client received server msg unexpectedly"); };
    let None = user2.next_val() else { panic!("client received server msg unexpectedly"); };
    let None = user3.next_val() else { panic!("client received server msg unexpectedly"); };
    let None = user4.next_val() else { panic!("client received server msg unexpectedly"); };
    let None = hub1.next_val() else { panic!("hub received server msg unexpectedly"); };
    let None = hub2.next_val() else { panic!("hub received server msg unexpectedly"); };


    // cleanup
    let _ = user1.close();
    let _ = user2.close();
    let _ = user3.close();
    let _ = user4.close();
    let _ = hub1.close();
    let _ = hub2.close();
    std::thread::sleep(Duration::from_millis(15));
}

//-------------------------------------------------------------------------------------------------------------------
