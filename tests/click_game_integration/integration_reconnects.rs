//local shortcuts
use bevy_girk_backend_public::*;
use bevy_girk_client_fw::*;
use bevy_girk_client_instance::ClientFactory;
use bevy_girk_client_instance::ClientInstanceReport;
use bevy_girk_game_fw::*;
use bevy_girk_game_hub_server::*;
use bevy_girk_game_instance::*;
use bevy_girk_host_server::*;
use bevy_girk_utils::*;
use crate::click_game_integration::*;
use crate::host_server::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_renet2::prelude::RenetClient;
use renet2_setup::*;

//standard shortcuts
use std::net::Ipv6Addr;
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn make_host_server_test_configs() -> HostServerStartupPack
{
    // configs
    let host_server_config = HostServerConfig{
            ticks_per_sec                   : None,  //we will manually update the host server
            ongoing_game_purge_period_ticks : 1u64,
        };
    let lobbies_cache_config = LobbiesCacheConfig{
            max_request_size: 10u16,
            lobby_checker: Box::new(ClickLobbyChecker{
                    max_lobby_players     : 2u16,
                    max_lobby_watchers    : 0u16,
                    min_players_to_launch : 2u16,
                }
            )
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

fn make_hub_server_test_configs() -> GameHubServerStartupPack
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

fn make_click_game_test_configs(game_ticks_per_sec: u32, game_num_ticks: u32) -> ClickGameFactoryConfig
{
    // versioning
    let test_protocol_id = get_test_protocol_id();

    // config
    let max_init_ticks  = 200;

    // server setup config
    let server_setup_config = GameServerSetupConfig{
            protocol_id  : test_protocol_id,
            expire_secs  : 10u64,
            timeout_secs : 1i32,  //very short for this test
            server_ip    : Ipv6Addr::LOCALHOST.into(),
            native_port  : 0,
            wasm_wt_port : 0,
            wasm_ws_port : 0,
            proxy_ip     : None,
            wss_certs    : None,
            ws_domain    : None,
            native_port_proxy: 0,
            wasm_ws_port_proxy: 0,
            wasm_wt_port_proxy: 0,
            has_wss_proxy: false,
        };

    // game framework config
    let game_fw_config = GameFwConfig::new(game_ticks_per_sec, max_init_ticks, 0);

    // game duration config
    let game_duration_config = GameDurationConfig::new(game_num_ticks);

    // click game factory config
    let game_factory_config = ClickGameFactoryConfig{
            server_setup_config,
            game_fw_config,
            game_duration_config,
        };

    game_factory_config
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn make_test_game_hub_server(
    hub_server_url      : url::Url,
    startup_pack        : GameHubServerStartupPack,
    game_factory_config : ClickGameFactoryConfig,
) -> (Sender<GameHubCommand>, App)
{
    // setup
    let (command_sender, command_receiver) = new_channel::<GameHubCommand>();
    let (_, host_hub_client)    = make_test_host_hub_client(hub_server_url);
    let game_launch_pack_source = GameLaunchPackSource::new(ClickGameLaunchPackSource::new(&game_factory_config));
    let game_factory            = GameFactory::new(ClickGameFactory{});
    let game_launcher           = GameInstanceLauncher::new(GameInstanceLauncherLocal::new(game_factory));

    // server app
    let server_app = make_game_hub_server(
            startup_pack,
            command_receiver,
            host_hub_client,
            game_launch_pack_source,
            game_launcher,
        );

    (command_sender, server_app)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn game_is_initialized(game_init_progress: Query<(&GameInitProgress, &StateScoped<ClientAppState>)>) -> bool
{
    let (progress, scoped) = game_init_progress.single().unwrap();
    tracing::debug!(?progress, "game init progress");
    assert_eq!(scoped.0, ClientAppState::Game);
    Readiness::new(**progress).is_ready()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn tick_clients_until_game_initialized(mut game_clients: Vec<&mut App>)
{
    // tick all the clients until initialized
    loop
    {
        // wait a bit for the game to update
        std::thread::sleep(Duration::from_millis(50));

        // update clients
        let mut num_inits = 0;

        for client in game_clients.iter_mut()
        {
            client.update();

            if *client.world().resource::<State<ClientInitState>>() != ClientInitState::Done
            { continue; }

            assert!(client.world().resource::<RenetClient>().is_connected());
            num_inits += 1;
        }

        // if not all clients are ready then we need to update them again
        if num_inits == game_clients.len() { break; }
    }

    // check that we have left the init phase as expected
    tracing::debug!("waiting for server to initialize clients");
    std::thread::sleep(Duration::from_millis(150));  //wait for server to finalize initialization

    for client in game_clients.iter_mut()
    {
        client.update();  //load game initialization progress entity changes
        assert!(client.world_mut().syscall((), game_is_initialized));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn launch_game(
    host_server: &mut App,
    hub_server: &mut App,
    user1: &mut HostUserClient,
    user2: &mut HostUserClient,
) -> (u64, App, App, GameStartInfo, GameStartInfo)
{
    // wait for everything to start up
    std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));

    let HostUserClientEvent::Report(_) = user1.next().unwrap() else { unimplemented!(); };
    let HostUserClientEvent::Report(_) = user2.next().unwrap() else { unimplemented!(); };

    // user 1 makes lobby
    user1.request(UserToHostRequest::MakeLobby{
            mcolor : ClickLobbyMemberType::Player.into(),
            pwd    : String::from("test"),
            data   : Vec::default()
        });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));

    // - user 1 recieves lobby
    let Some(HostUserClientEvent::Response(HostToUserResponse::LobbyJoin{ lobby }, _)) = user1.next()
    else { panic!("client did not receive server msg"); };
    let made_lobby_id = lobby.id;


    // user 2 joins lobby
    user2.request(UserToHostRequest::JoinLobby{
            id     : made_lobby_id,
            mcolor : ClickLobbyMemberType::Player.into(),
            pwd    : String::from("test")
        });
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
    user1.request(UserToHostRequest::LaunchLobbyGame{ id: lobby.id });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));

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
    user1.send(UserToHostMsg::AckPendingLobby{ id });
    user2.send(UserToHostMsg::AckPendingLobby{ id });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));


    // wait for game start report to percolate
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(45));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));

    // - users 1, 2 receive game start
    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameStart{ id, token: token1, start: start1 })) = user1.next()
    else { panic!("client did not receive server msg"); };
    let game_id = id;

    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameStart{ id, token: token2, start: start2 })) = user2.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, game_id);


    // users 1, 2 make game clients
    // - we only make the cores here, no client skins
    let mut client_factory1 = ClientFactory::new(ClickClientFactory::new(get_test_protocol_id()));
    let mut client_factory2 = ClientFactory::new(ClickClientFactory::new(get_test_protocol_id()));
    let mut client_app1 = App::new();
    let mut client_app2 = App::new();
    client_factory1.add_plugins(&mut client_app1);
    client_factory2.add_plugins(&mut client_app2);
    client_factory1.setup_game(client_app1.world_mut(), token1, start1.clone());
    client_factory2.setup_game(client_app2.world_mut(), token2, start2.clone());
    client_app1.insert_resource(client_factory1);
    client_app2.insert_resource(client_factory2);


    // tick clients until the game is fully initialized
    tick_clients_until_game_initialized(vec![&mut client_app1, &mut client_app2]);

    (game_id, client_app1, client_app2, start1, start2)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn game_end_cleanup(
    mut host_server: App,
    mut hub_server: App,
    mut user1: HostUserClient,
    mut user2: HostUserClient,
    mut client_app1: App,
    mut client_app2: App,
    game_id: u64,
){
    // wait for game over
    let mut report1: Option<GameOverReport> = None;
    let mut report2: Option<GameOverReport> = None;

    loop
    {
        // update everything
        host_server.update(); hub_server.update(); client_app1.update(); client_app2.update();
        std::thread::sleep(Duration::from_millis(15));

        // look for game over reports from host server
        if let Some(HostUserClientEvent::Msg(HostToUserMsg::GameOver{ id, report })) = user1.next()
        {
            assert_eq!(id, game_id);
            report1 = Some(report);
        }
        if let Some(HostUserClientEvent::Msg(HostToUserMsg::GameOver{ id, report })) = user2.next()
        {
            assert_eq!(id, game_id);
            report2 = Some(report);
        }

        // we are done when both users have game over reports
        if report1.is_some() && report2.is_some() { break; }
    }


    // - users 1, 2 receive nothing else
    let None = user1.next() else { panic!("client received server msg unexpectedly"); };
    let None = user2.next() else { panic!("client received server msg unexpectedly"); };


    // cleanup
    let _ = user1.close();
    let _ = user2.close();
    std::thread::sleep(Duration::from_millis(15));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

// Reconnect by replacing the game client.
#[test]
fn integration_reconnect_gameclient_restart()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // launch host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_host_server_test_configs());

    // launch game hub server attached to host server
    let game_ticks_per_sec = 20;
    let game_num_ticks     = 30;  //may need to increase this if test is failing
    let (_hub_command_sender, mut hub_server) = make_test_game_hub_server(
            host_hub_url,
            make_hub_server_test_configs(),
            make_click_game_test_configs(game_ticks_per_sec, game_num_ticks)
        );

    // make user clients
    let user1_id = 0u128;
    let user2_id = 1u128;
    let (_, mut user1) = make_test_host_user_client_with_id(user1_id, host_user_url.clone());
    let (_, mut user2) = make_test_host_user_client_with_id(user2_id, host_user_url.clone());


    // launch game
    let (game_id, client_app1, mut client_app2, start1, _) = launch_game(&mut host_server, &mut hub_server, &mut user1, &mut user2);


    // disconnect game client 1
    //TODO: This DOES NOT WORK unless the delay after disconnecting is large enough, because server and client currently
    //      have the same connection timeout, so the server will timeout the old connect at the same time the new
    //      connection attempt times out.
    std::mem::drop(client_app1);
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(145));

    // request new connect token for client 1
    user1.request(UserToHostRequest::GetConnectToken{ id: game_id });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));

    // - receive new connect token
    let Some(HostUserClientEvent::Response(HostToUserResponse::ConnectToken{ id, token }, _)) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, game_id);


    // remake game client 1
    let mut client_factory = ClientFactory::new(ClickClientFactory::new(get_test_protocol_id()));
    let mut client_app1 = App::new();
    client_factory.add_plugins(&mut client_app1);
    client_factory.setup_game(client_app1.world_mut(), token, start1);


    // tick clients until the game is fully initialized for the reconnected client
    tick_clients_until_game_initialized(vec![&mut client_app1, &mut client_app2]);


    // cleanup
    game_end_cleanup(host_server, hub_server, user1, user2, client_app1, client_app2, game_id);
}

//-------------------------------------------------------------------------------------------------------------------

// Reconnect while the game app is still alive.
#[test]
fn integration_reconnect_gameclient_reconnect()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // launch host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_host_server_test_configs());

    // launch game hub server attached to host server
    let game_ticks_per_sec = 20;
    let game_num_ticks     = 30;
    let (_hub_command_sender, mut hub_server) = make_test_game_hub_server(
            host_hub_url,
            make_hub_server_test_configs(),
            make_click_game_test_configs(game_ticks_per_sec, game_num_ticks)
        );

    // make user clients
    let user1_id = 0u128;
    let user2_id = 1u128;
    let (_, mut user1) = make_test_host_user_client_with_id(user1_id, host_user_url.clone());
    let (_, mut user2) = make_test_host_user_client_with_id(user2_id, host_user_url.clone());


    // launch game
    let (game_id,mut client_app1,mut client_app2, start1, _) = launch_game(&mut host_server, &mut hub_server, &mut user1, &mut user2);


    // disconnect game client 1 from renet server
    assert_eq!(*client_app1.world().resource::<State<ClientAppState>>().get(), ClientAppState::Game);
    client_app1.world_mut().resource_mut::<RenetClient>().disconnect();
    client_app1.update();
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(45));
    client_app1.update(); std::thread::sleep(Duration::from_millis(45));
    assert!(client_app1.world().get_resource::<RenetClient>().is_none());
    assert!(client_app1.world().get_resource::<State<ClientInitState>>().is_none());
    assert_eq!(*client_app1.world().resource::<State<ClientAppState>>().get(), ClientAppState::Client);
    
    // request new connect token for client 1
    let Some(ClientInstanceReport::RequestConnectToken(req_id)) = client_app1.world_mut().resource_mut::<Events<ClientInstanceReport>>().drain().next() else {
        panic!("client did not receive connect token req");
    };
    assert_eq!(req_id, game_id);
    user1.request(UserToHostRequest::GetConnectToken{ id: game_id });
    std::thread::sleep(Duration::from_millis(15));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(15));

    // - receive new connect token
    let Some(HostUserClientEvent::Response(HostToUserResponse::ConnectToken{ id, token }, _)) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(id, game_id);


    // reconnect game client app
    client_app1.world_mut().resource_scope(|world: &mut World, mut factory: Mut<ClientFactory>| {
        factory.setup_game(world, token, start1);
    });

    // tick clients until the game is fully initialized for the reconnected client
    tick_clients_until_game_initialized(vec![&mut client_app1, &mut client_app2]);


    // cleanup
    game_end_cleanup(host_server, hub_server, user1, user2, client_app1, client_app2, game_id);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn integration_reconnect_userclient_restart()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // launch host server
    let (mut host_server, host_hub_url, host_user_url) = make_test_host_server(make_host_server_test_configs());

    // launch game hub server attached to host server
    let game_ticks_per_sec = 20;
    let game_num_ticks     = 40;
    let (_hub_command_sender, mut hub_server) = make_test_game_hub_server(
            host_hub_url,
            make_hub_server_test_configs(),
            make_click_game_test_configs(game_ticks_per_sec, game_num_ticks)
        );

    // make user clients
    let user1_id = 0u128;
    let user2_id = 1u128;
    let (user1_id, mut user1) = make_test_host_user_client_with_id(user1_id, host_user_url.clone());
    let (_, mut user2)        = make_test_host_user_client_with_id(user2_id, host_user_url.clone());


    // launch game
    let (game_id, _, mut client_app2, _, _) = launch_game(&mut host_server, &mut hub_server, &mut user1, &mut user2);


    // disconnect client 1
    // - we do this *before* the server times out the connection to make sure rapid reconnects work
    user1.close();
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(45));

    // reconnect client 1
    let (user1_id, mut user1) = make_test_host_user_client_with_id(user1_id, host_user_url.clone());
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(45));
    host_server.update(); hub_server.update(); std::thread::sleep(Duration::from_millis(45));

    let HostUserClientEvent::Report(_) = user1.next().unwrap() else { unimplemented!(); };

    // receive game start on reconnect
    let Some(HostUserClientEvent::Msg(HostToUserMsg::GameStart{ id, token: token1, start: start1 })) = user1.next()
    else { panic!("client did not receive server msg"); };
    assert_eq!(user1_id, start1.user_id);
    assert_eq!(id, game_id);

    // remake game client 1
    let mut client_factory = ClientFactory::new(ClickClientFactory::new(get_test_protocol_id()));
    let mut client_app1 = App::new();
    client_factory.add_plugins(&mut client_app1);
    client_factory.setup_game(client_app1.world_mut(), token1, start1);


    // tick clients until the game is fully initialized for the reconnected client
    // NOTE: this can fail if the game ends before the client reconnects (race condition)
    tick_clients_until_game_initialized(vec![&mut client_app1, &mut client_app2]);


    // cleanup
    game_end_cleanup(host_server, hub_server, user1, user2, client_app1, client_app2, game_id);
}

//-------------------------------------------------------------------------------------------------------------------
