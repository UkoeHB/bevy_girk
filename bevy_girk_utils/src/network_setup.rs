//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::syscall;
use bevy_renet2::renet2::{ChannelConfig, ConnectionConfig, RenetClient, RenetServer};
use bevy_renet2::renet2::transport::{
    ClientAuthentication, NativeSocket, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
    ServerSetupConfig,
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

fn create_server(
    server_channels_config : Vec<ChannelConfig>,
    client_channels_config : Vec<ChannelConfig>,
    mut server_config      : ServerSetupConfig
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
    let server_socket = UdpSocket::bind(server_config.socket_addresses[0][0])
        .expect("renet server address should be bindable");
    server_config.socket_addresses = vec![vec![server_socket.local_addr().unwrap()]];

    // make transport
    let server_transport = NetcodeServerTransport::new(server_config, NativeSocket::new(server_socket).unwrap()).unwrap();

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
    let client_transport = NetcodeClientTransport::new(
        current_time,
        authentication, NativeSocket::new(client_socket).unwrap()
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
                socket_id: 0,
                server_addr,
                user_data: None,
            };

    // finish making client
    create_client(server_channels_config, client_channels_config, authentication, client_addr)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Sets up a renet client with default transport using the provided authentication and client address.
/// - Assumes there is a `bevy_replicon::RepliconChannels` resource already loaded in the app.
fn setup_native_renet_client(
    In((
        authentication,
        client_address
    ))                      : In<(ClientAuthentication, SocketAddr)>,
    mut client_app_commands : Commands,
    replicon_channels       : Res<RepliconChannels>,
){
    // get server/client channels
    let server_channels = replicon_channels.get_server_configs();
    let client_channels = replicon_channels.get_client_configs();

    // make server
    let (client, client_transport) = create_client(
            server_channels.clone(),
            client_channels.clone(),
            authentication,
            client_address,
        );

    // add client and transport
    client_app_commands.insert_resource(client);
    client_app_commands.insert_resource(client_transport);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Set up a renet server with default transport using the provided `ServerConfig`.
/// - Assumes there is a bevy_replicon::RepliconChannels resource already loaded in the app.
pub fn setup_native_renet_server(server_app: &mut App, server_config: ServerSetupConfig) -> SocketAddr
{
    tracing::info!("setting up renet server");

    // get server/client channels
    let replicon_channels = server_app.world.resource::<RepliconChannels>();
    let server_channels   = replicon_channels.get_server_configs();
    let client_channels   = replicon_channels.get_client_configs();

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
///
/// Connect packs should be considered single-use. If you need to reconnect, make a new connect pack with fresh
/// connection authentication.
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

impl RenetClientConnectPack
{
    /// Make a new connect pack from a server connect token.
    pub fn new(expected_protocol_id: u64, token: ServerConnectToken) -> Result<Self, String>
    {
        // extract connect token
        let ServerConnectToken::Native{ bytes } = token;
        //else { panic!("only native connect tokens currently supported"); };

        let connect_token = connect_token_from_bytes(&bytes)
            .map_err(|_| String::from("failed deserializing renet connect token"))?;

        // check protocol version
        if connect_token.protocol_id != expected_protocol_id { return Err(String::from("protocol id mismatch")); }

        // prepare client address based on server address
        let Some(server_addr) = connect_token.server_addresses[0]
        else { return Err(String::from("server address is missing")); };
        let client_address = client_address_from_server_address(&server_addr);

        Ok(Self::Native(ClientAuthentication::Secure{ connect_token }, client_address))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a renet client with pre-loaded connection information.
/// - Removes the [`RenetClientConnectPack`] resource from the world, or returns an error if it is missing.
pub fn setup_renet_client(world: &mut World) -> Result<(), ()>
{
    tracing::info!("setting up renet client");

    let connect_pack = world.remove_resource::<RenetClientConnectPack>().ok_or(())?;
    match connect_pack
    {
        RenetClientConnectPack::Native(authentication, client_address) =>
        {
            // drop the existing transport to free its address in case we are re-using a client address
            world.remove_resource::<NetcodeClientTransport>();

            // setup client
            syscall(world, (authentication, client_address), setup_native_renet_client);
        }
    }

    Ok(())
}

//-------------------------------------------------------------------------------------------------------------------

/// Automates server and client creation for a local test server. Assumes app and clients are in separate bevy Apps.
///
/// Requires:
/// - All apps need `bevy_replicon::prelude::RepliconCorePlugin`.
/// - Must be called after ALL server and client channels have been set up.
pub fn setup_local_test_renet_network(server_app: &mut App, client_apps: &mut Vec<App>)
{
    // get server/client channels
    let replicon_channels = server_app.world.resource::<RepliconChannels>();
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
