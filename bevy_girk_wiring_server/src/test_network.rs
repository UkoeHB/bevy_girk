//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use renet2::{ChannelConfig, ConnectionConfig, RenetClient, RenetServer};
use renet2_netcode::{
    ClientAuthentication, NativeSocket, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
    ServerSetupConfig, ServerSocket
};
use bevy_replicon::core::channels::RepliconChannels;
use bevy_replicon_renet2::RenetChannelsExt;

//standard shortcuts
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

const LOCALHOST_TEST_NETWORK_PROTOCOL_ID: u64 = 0;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn create_test_client_native(
    server_channels_config : Vec<ChannelConfig>,
    client_channels_config : Vec<ChannelConfig>,
    authentication         : ClientAuthentication,
    client_addr            : SocketAddr,
) -> (RenetClient, NetcodeClientTransport)
{
    // make client
    let client_socket = NativeSocket::new(
        UdpSocket::bind(client_addr).expect("renet client address should be bindable")
    ).unwrap();
    let client = RenetClient::new(
        ConnectionConfig::from_channels(
            server_channels_config,
            client_channels_config,
        ),
        client_socket.is_reliable(),
    );

    // make transport
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_transport = NetcodeClientTransport::new(
        current_time,
        authentication,
        client_socket
    ).unwrap();

    (client, client_transport)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn create_localhost_test_server(
    server_channels_config : Vec<ChannelConfig>,
    client_channels_config : Vec<ChannelConfig>,
    num_clients            : u16
) -> (RenetServer, NetcodeServerTransport)
{
    // server config
    let server_config =
        ServerSetupConfig{
                current_time     : SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                max_clients      : num_clients as usize,
                protocol_id      : LOCALHOST_TEST_NETWORK_PROTOCOL_ID,
                socket_addresses : vec![vec![SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0)]],
                authentication   : ServerAuthentication::Unsecure,
            };

    // finish making server
    create_native_server(server_channels_config, client_channels_config, server_config)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn create_localhost_test_client(
    server_addr            : SocketAddr,
    server_channels_config : Vec<ChannelConfig>,
    client_channels_config : Vec<ChannelConfig>,
    client_id              : u64
) -> (RenetClient, NetcodeClientTransport)
{
    // authentication
    let client_ip      = Ipv4Addr::LOCALHOST.into();
    let client_addr    = SocketAddr::new(client_ip, 0);
    let authentication =
        ClientAuthentication::Unsecure{
                client_id,
                protocol_id: LOCALHOST_TEST_NETWORK_PROTOCOL_ID,
                socket_id: 0,
                server_addr,
                user_data: None,
            };

    // finish making client
    create_test_client_native(server_channels_config, client_channels_config, authentication, client_addr)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Automates server and client creation for a local test server. Assumes app and clients are in separate bevy Apps.
///
/// Requires:
/// - All apps need `bevy_replicon::prelude::RepliconCorePlugin`.
/// - Must be called after ALL server and client channels have been set up.
pub fn setup_local_test_renet_network(server_app: &mut App, client_apps: &mut Vec<App>)
{
    // get server/client channels
    let replicon_channels = server_app.world().resource::<RepliconChannels>();
    let server_channels   = replicon_channels.get_server_configs();
    let client_channels   = replicon_channels.get_client_configs();

    // make server
    let (server, server_transport) = create_localhost_test_server(
            server_channels.clone(),
            client_channels.clone(),
            client_apps.len() as u16
        );
    let server_addr = server_transport.addresses()[0];
    server_app
        .insert_resource(server)
        .insert_resource(server_transport);

    // make clients
    for (index, client_app) in client_apps.iter_mut().enumerate()
    {
        let (client, client_transport) = create_localhost_test_client(
                server_addr,
                server_channels.clone(),
                client_channels.clone(),
                index as u64
            );
        client_app
            .insert_resource(client)
            .insert_resource(client_transport);
    }
}

//-------------------------------------------------------------------------------------------------------------------
