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
fn lobby_owner_invalid()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs(0u16));

    // make a game hub client
    let (_, hub) = make_test_host_hub_client(host_hub_url);

    // make user clients
    let (_, user1) = make_test_host_user_client(host_user_url);


    // hub initializes its capacity
    hub.send(HubToHostMsg::Capacity(GameHubCapacity(1))).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));


    // user 1 makes lobby
    let request = user1.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 recieves nothing (make lobby failed because no players are allowed in the lobby)
    let Some(HostUserServerVal::Reject(id)) = user1.next_val()
    else { panic!("client did not receive server message"); };
    assert_eq!(id, request.id());


    // nothing more to receive
    let None = user1.next_val() else { panic!("client received server msg unexpectedly"); };
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn lobby_data_invalid()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs(1u16));

    // make a game hub client
    let (_, hub) = make_test_host_hub_client(host_hub_url);

    // make user client
    let (_, user1) = make_test_host_user_client(host_user_url);


    // hub initializes its capacity
    hub.send(HubToHostMsg::Capacity(GameHubCapacity(1))).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));


    // user 1 makes lobby
    let request = user1.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : vec![1u8]   //invalid data
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 recieves request rejection (make lobby failed because no custom data is allowed)
    let Some(HostUserServerVal::Reject(id)) = user1.next_val()
    else { panic!("client did not receive server message"); };
    assert_eq!(id, request.id());

    
    // no more incoming
    let None = user1.next_val() else { panic!("client received server msg unexpectedly"); };
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn lobby_member_invalid_type()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs(10u16));

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
            data   : Vec::default(),
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 recieves lobby
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ id: made_lobby_id, lobby: _ }, _)) = user1.next_val()
    else { panic!("client did not receive server msg"); };


    // user 2 joins lobby
    let request = user2.request(UserToHostRequest::JoinLobby{
            id     : made_lobby_id,
            mcolor : BasicLobbyMemberType::Watcher.into(),
            pwd    : String::from("test")
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives nothing (no watchers allowed)
    let Some(HostUserServerVal::Reject(id)) = user2.next_val()
    else { panic!("client did not receive server message"); };
    assert_eq!(id, request.id());


    // nothing more to receive
    let None = user2.next_val() else { panic!("client received server msg unexpectedly"); };
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn lobby_member_invalid_password()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_configs(10u16));

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
            data   : Vec::default(),
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 1 recieves lobby
    let Some(HostUserServerVal::Response(HostToUserResponse::LobbyJoin{ id: made_lobby_id, lobby: _ }, _)) = user1.next_val()
    else { panic!("client did not receive server msg"); };


    // user 2 joins lobby
    let request = user2.request(UserToHostRequest::JoinLobby{
            id     : made_lobby_id,
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("invalid")
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update();
    std::thread::sleep(Duration::from_millis(15));

    // - user 2 receives nothing (bad password)
    let Some(HostUserServerVal::Reject(id)) = user2.next_val()
    else { panic!("client did not receive server message"); };
    assert_eq!(id, request.id());


    // nothing more to receive
    let None = user2.next_val() else { panic!("client received server msg unexpectedly"); };
}

//-------------------------------------------------------------------------------------------------------------------
