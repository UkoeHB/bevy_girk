//local shortcuts
use crate::game_hub_server::*;
use crate::host_server::*;
use crate::test_helpers::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_hub_server::*;
use bevy_girk_host_server::*;

//third-party shortcuts

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn make_host_server_configs() -> HostServerStartupPack
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
            expiry_duration: Duration::from_secs(1),
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

fn make_hub_server_configs() -> GameHubServerStartupPack
{
    let game_hub_server_config = GameHubServerConfig{
            ticks_per_sec                   : None,  //we will manually update the game hub server
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
fn basic_host_hub_integration()
{
    // launch host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_host_server_configs());

    // launch game hub server attached to host server
    let game_ticks_per_sec = Ticks(100);
    let game_num_ticks     = Ticks(3);
    let (_hub_command_sender, mut hub_server) = make_test_game_hub_server(
            host_hub_url,
            false,
            make_hub_server_configs(),
            game_ticks_per_sec,
            game_num_ticks,
            Some(true)
        );

    // make user client
    let (_user1_id, user1) = make_test_host_user_client(host_user_url);


    // wait for everything to start up
    std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));

    let HostUserClientEvent::Report(_) = user1.next().unwrap() else { unimplemented!(); };

    // user 1 makes lobby
    user1.request(UserToHostRequest::MakeLobby{
            mcolor : BasicLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));

    // - user 1 recieves lobby
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user1.next()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id = lobby.id;


    // user 1 launches lobby
    user1.request(UserToHostRequest::LaunchLobbyGame{ id: made_lobby_id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));

    // - user 1 receives ack request
    let Some(HostUserClientEvent::Msg(HostToUserMsg::PendingLobbyAckRequest{ id })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);

    // - user 1 receives ack for launching the game
    let Some(HostUserClientEvent::Ack(_request_id)) = user1.next()
    else { panic!("client did not receive server msg"); };


    // user 1 sends ack
    user1.send(UserToHostMsg::AckPendingLobby{ id }).expect("send failed");
    std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(45));


    // wait for game start report to percolate
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(45));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));


    // get game start report
    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameStart{ id: _, connect: _ , start: _})) = user1.next()
    else { panic!("client did not receive server msg"); };


    // wait for game over
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));

    // - get game over from host server
    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameOver{ id, report: _ })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, made_lobby_id);


    // - user receives nothing else
    let None = user1.next() else { panic!("client received server msg unexpectedly"); };


    // cleanup
    let _ = user1.close();
    std::thread::sleep(Duration::from_millis(15));
}

//-------------------------------------------------------------------------------------------------------------------
