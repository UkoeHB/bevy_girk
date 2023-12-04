//local shortcuts
use bevy_kot_ecs::syscall;

//third-party shortcuts
use bevy::prelude::*;
use bevy_renet::renet::{ChannelConfig, ConnectionConfig, RenetClient, RenetServer};
use bevy_renet::renet::transport::{
    ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
    ServerConfig,
};
use bevy_replicon::prelude::NetworkChannels;

//standard shortcuts
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

const LOCALHOST_TEST_NETWORK_PROTOCOL_ID: u64 = 0;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn create_server(
    server_channels_config : Vec<ChannelConfig>,
    client_channels_config : Vec<ChannelConfig>,
    mut server_config      : ServerConfig
) -> (RenetServer, NetcodeServerTransport)
{
    // make server
    let server = RenetServer::new(
            ConnectionConfig{
                    server_channels_config,
                    client_channels_config,
                    ..default()
                }
        );

    // prepare udp socket
    // - finalizes the public address (wildcards should be resolved)
    let server_socket = UdpSocket::bind(server_config.public_addresses[0])
        .expect("renet server address should be bindable");
    server_config.public_addresses = vec![server_socket.local_addr().unwrap()];

    // make transport
    let server_transport = NetcodeServerTransport::new(server_config, server_socket).unwrap();

    (server, server_transport)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn create_client(
    server_channels_config : Vec<ChannelConfig>,
    client_channels_config : Vec<ChannelConfig>,
    authentication         : ClientAuthentication,
    client_addr            : SocketAddr,
) -> (RenetClient, NetcodeClientTransport)
{
    // make client
    let client = RenetClient::new(
            ConnectionConfig{
                    server_channels_config,
                    client_channels_config,
                    ..default()
                }
        );

    // make transport
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_socket = UdpSocket::bind(client_addr).expect("renet client address should be bindable");
    let client_transport = NetcodeClientTransport::new(current_time, authentication, client_socket).unwrap();

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
        ServerConfig{
                current_time     : SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                max_clients      : num_clients as usize,
                protocol_id      : LOCALHOST_TEST_NETWORK_PROTOCOL_ID,
                public_addresses : vec![SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0)],
                authentication   : ServerAuthentication::Unsecure,
            };

    // finish making server
    create_server(server_channels_config, client_channels_config, server_config)
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
                server_addr,
                user_data: None,
            };

    // finish making client
    create_client(server_channels_config, client_channels_config, authentication, client_addr)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Sets up a renet client with default transport using the provided authentication and client address.
/// - Assumes there is a `bevy_replicon::NetworkChannels` resource already loaded in the app.
fn setup_native_renet_client(
    In((
        authentication,
        client_address
    ))                      : In<(ClientAuthentication, SocketAddr)>,
    mut client_app_commands : Commands,
    network_channels        : Res<NetworkChannels>,
){
    // get server/client channels
    let server_channels  = network_channels.get_server_configs();
    let client_channels  = network_channels.get_client_configs();

    // make server
    let (client, client_transport) = create_client(
            server_channels.clone(),
            client_channels.clone(),
            authentication,
            client_address,
        );

    // add client and transport
    // - this will over-write any preexisting client/transport (useful for when a client is disconnected)
    client_app_commands.insert_resource(client);
    client_app_commands.insert_resource(client_transport);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Set up a renet server with default transport using the provided `ServerConfig`.
/// - Assumes there is a bevy_replicon::NetworkChannels resource already loaded in the app.
pub fn setup_native_renet_server(server_app: &mut App, server_config: ServerConfig) -> SocketAddr
{
    // get server/client channels
    let network_channels = server_app.world.resource::<NetworkChannels>();
    let server_channels  = network_channels.get_server_configs();
    let client_channels  = network_channels.get_client_configs();

    // make server
    let (server, server_transport) = create_server(
            server_channels,
            client_channels,
            server_config,
        );

    // add server to app
    let server_addr = server_transport.addresses()[0];
    server_app
        .insert_resource(server)
        .insert_resource(server_transport);

    server_addr
}

//-------------------------------------------------------------------------------------------------------------------

/// Information needed to connect a renet client to a renet server.
///
/// Add this as a resource to your app before trying to set up a renet client.
#[derive(Resource, Debug, Clone)]
pub enum RenetClientConnectPack
{
    /// Connection information for native transports.
    /// 
    /// Note: The client address should be tailored to the server address type (Ipv4/Ipv6).
    Native(ClientAuthentication, SocketAddr),
    //Wasm,
    //Local,
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a renet client with pre-loaded connection information.
/// - Assumes there is a [`RenetClientConnectPack`] resource already loaded in the app.
pub fn setup_renet_client(world: &mut World)
{
    let connect_pack = world.resource::<RenetClientConnectPack>().clone();
    match connect_pack
    {
        RenetClientConnectPack::Native(authentication, client_addr) =>
        {
            syscall(world, (authentication, client_addr), setup_native_renet_client);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Automates server and client creation for a local test server. Assumes app and clients are in separate bevy Apps.
///
/// Requires:
/// - All apps need `bevy_replicon::ReplicationCorePlugin`.
/// - Must be called after ALL server and client channels have been set up.
pub fn setup_local_test_renet_network(server_app: &mut App, client_apps: &mut Vec<App>)
{
    // get server/client channels
    let network_channels = server_app.world.resource::<NetworkChannels>();
    let server_channels  = network_channels.get_server_configs();
    let client_channels  = network_channels.get_client_configs();

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
