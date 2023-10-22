//local shortcuts
use crate::test_helpers::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_hub_server::*;
use bevy_girk_game_instance::*;
use bevy_girk_host_server::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn make_test_host_hub_client_with_id(
    client_id                 : u128,
    hub_server_url            : url::Url,
    reconnect_on_server_close : bool,
) -> HostHubClient
{
    let auth = bevy_simplenet::AuthRequest::None{ client_id };

    host_hub_client_factory().new_client(
            enfync::builtin::native::TokioHandle::default(),
            hub_server_url,
            auth,
            bevy_simplenet::ClientConfig{
                    reconnect_on_disconnect: true,
                    reconnect_on_server_close,
                    ..Default::default()
                },
            ()
        )
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub fn make_test_game_hub_server(
    hub_server_url            : url::Url,
    reconnect_on_server_close : bool,
    startup_pack              : GameHubServerStartupPack,
    game_ticks_per_sec        : Ticks,
    game_num_ticks            : Ticks,
    lp_source_works           : Option<bool>,
) -> (MessageSender<GameHubCommand>, App)
{
    // misc
    let (command_sender, command_receiver) = new_message_channel::<GameHubCommand>();
    let host_hub_client = make_test_host_hub_client_with_id(0u128, hub_server_url, reconnect_on_server_close);
    let game_factory    = GameFactory::new(DummyGameFactory{});
    let game_launcher   = GameInstanceLauncher::new(GameInstanceLauncherLocal::new(game_factory));

    // game config
    let game_config = DummyGameConfig{
            ticks_per_sec       : game_ticks_per_sec,
            game_duration_ticks : game_num_ticks,
        };
    let game_config_ser = ser_msg(&game_config);
    let game_launch_pack_source = GameLaunchPackSource::new(
            DummyGameLaunchPackSource::new(game_config_ser, lp_source_works)
        );

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

pub fn make_test_host_hub_server() -> HostHubServer
{
    // host-hub server
    host_hub_server_factory().new_server(
            enfync::builtin::Handle::default(),
            "127.0.0.1:0",
            bevy_simplenet::AcceptorConfig::Default,
            bevy_simplenet::Authenticator::None,
            bevy_simplenet::ServerConfig{
                max_connections   : 10,
                max_msg_size      : 10_000,
                rate_limit_config : bevy_simplenet::RateLimitConfig{
                        period    : Duration::from_millis(15),
                        max_count : 500
                    },
                ..Default::default()
            }
        )
}

//-------------------------------------------------------------------------------------------------------------------
