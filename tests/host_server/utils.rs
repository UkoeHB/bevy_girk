//local shortcuts
use bevy_girk_backend_public::*;
use bevy_girk_game_instance::*;
use bevy_girk_host_server::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------

pub fn make_test_host_server(configs: HostServerStartupPack) -> (App, url::Url, url::Url)
{
    // host-hub server
    let host_hub_server = host_hub_server_factory().new_server(
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
        );
    let host_hub_url = host_hub_server.url();

    // host-user server
    let host_user_server = host_user_server_factory().new_server(
            enfync::builtin::Handle::default(),
            "127.0.0.1:0",
            bevy_simplenet::AcceptorConfig::Default,
            bevy_simplenet::Authenticator::None,
            bevy_simplenet::ServerConfig{
                max_connections   : 10,
                max_msg_size      : 10_000,
                rate_limit_config : bevy_simplenet::RateLimitConfig{
                        period    : Duration::from_millis(15),
                        max_count : 25
                    },
                ..Default::default()
            }
        );
    let host_user_url = host_user_server.url();

    (
        make_host_server(configs, host_hub_server, host_user_server),
        host_hub_url,
        host_user_url,
    )
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_test_host_hub_client_with_id(id: u128, hub_server_url: url::Url) -> (u128, HostHubClient)
{
    let auth = bevy_simplenet::AuthRequest::None{ client_id: id };

    (
        id,
        host_hub_client_factory().new_client(
                enfync::builtin::Handle::default(),
                hub_server_url,
                auth,
                bevy_simplenet::ClientConfig::default(),
                ()
            )
    )
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_test_host_hub_client(hub_server_url: url::Url) -> (u128, HostHubClient)
{
    make_test_host_hub_client_with_id(gen_rand128(), hub_server_url)
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_test_host_user_client_with_id(id: u128, user_server_url: url::Url) -> (u128, HostUserClient)
{
    let auth = bevy_simplenet::AuthRequest::None{ client_id: id };

    (
        id,
        host_user_client_factory().new_client(
                enfync::builtin::Handle::default(),
                user_server_url,
                auth,
                bevy_simplenet::ClientConfig::default(),
                ()
            )
    )
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_test_host_user_client(user_server_url: url::Url) -> (u128, HostUserClient)
{
    make_test_host_user_client_with_id(gen_rand128(), user_server_url)
}

//-------------------------------------------------------------------------------------------------------------------

pub fn dummy_game_start_report(user_ids: Vec<u128>) -> GameStartReport
{
    let mut connect_infos = Vec::default();
    for user_id in user_ids.iter()
    {
        connect_infos.push(GameConnectInfo { user_id: *user_id, ..default() });
    }

    GameStartReport{ connect_infos }
}

//-------------------------------------------------------------------------------------------------------------------
