//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_renet2::prelude::{ChannelConfig, ConnectionConfig, RenetClient};
use bevy_renet2::netcode::{
    ClientAuthentication, ClientSocket, NativeSocket, NetcodeClientTransport
};
use bevy_replicon::core::channels::RepliconChannels;
use bevy_replicon_renet2::RenetChannelsExt;

//standard shortcuts
use std::net::{SocketAddr, UdpSocket};
use wasm_timer::SystemTime;

use crate::ClientConnectPack;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn create_native_client(
    server_channels_config : Vec<ChannelConfig>,
    client_channels_config : Vec<ChannelConfig>,
    authentication         : ClientAuthentication,
    client_addr            : SocketAddr,
) -> (RenetClient, NetcodeClientTransport)
{
    // make client
    let udp_socket = UdpSocket::bind(client_addr).expect("renet client address should be bindable");
    let client_socket = NativeSocket::new(udp_socket).unwrap();
    let client = RenetClient::new(
        ConnectionConfig::from_channels(server_channels_config, client_channels_config),
        client_socket.is_reliable()
    );

    // make transport
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_transport = NetcodeClientTransport::new(
        current_time,
        authentication,
        client_socket,
    ).unwrap();

    (client, client_transport)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

// Note: this depends on bevy_renet2/wt_client_transport feature.
#[cfg(target_family = "wasm")]
fn create_wasm_wt_client(
    server_channels_config : Vec<ChannelConfig>,
    client_channels_config : Vec<ChannelConfig>,
    authentication         : ClientAuthentication,
    config                 : bevy_renet2::netcode::WebTransportClientConfig,
) -> (RenetClient, NetcodeClientTransport)
{
    // make client
    let client_socket = bevy_renet2::netcode::WebTransportClient::new(config);
    let client = RenetClient::new(
        ConnectionConfig{
            server_channels_config,
            client_channels_config,
            ..default()
        },
        client_socket.is_reliable()
    );

    // make transport
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_transport = NetcodeClientTransport::new(
        current_time,
        authentication,
        client_socket,
    ).unwrap();

    (client, client_transport)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

// Note: this depends on bevy_renet2/ws_client_transport feature.
#[cfg(target_family = "wasm")]
fn create_wasm_ws_client(
    server_channels_config : Vec<ChannelConfig>,
    client_channels_config : Vec<ChannelConfig>,
    authentication         : ClientAuthentication,
    config                 : bevy_renet2::netcode::WebSocketClientConfig,
) -> (RenetClient, NetcodeClientTransport)
{
    // make client
    let client_socket = bevy_renet2::netcode::WebSocketClient::new(config);
    let client = RenetClient::new(
        ConnectionConfig{
            server_channels_config,
            client_channels_config,
            ..default()
        },
        client_socket.is_reliable()
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

#[cfg(feature = "memory_transport")]
fn create_memory_client(
    server_channels_config: Vec<ChannelConfig>,
    client_channels_config: Vec<ChannelConfig>,
    authentication: ClientAuthentication,
    client_socket: bevy_renet2::netcode::MemorySocketClient,
) -> (RenetClient, NetcodeClientTransport)
{
    // make client
    let client = RenetClient::new(
        ConnectionConfig::from_channels(server_channels_config, client_channels_config),
        client_socket.is_reliable()
    );

    // make transport
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_transport = NetcodeClientTransport::new(current_time, authentication, client_socket).unwrap();

    (client, client_transport)
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
    let (client, client_transport) = create_native_client(
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

/// Sets up a renet client with wasm webtransport transport using the provided authentication and client address.
/// - Assumes there is a `bevy_replicon::RepliconChannels` resource already loaded in the app.
#[cfg(target_family = "wasm")]
fn setup_wasm_wt_renet_client(
    In((
        authentication,
        config
    ))                      : In<(ClientAuthentication, bevy_renet2::netcode::WebTransportClientConfig)>,
    mut client_app_commands : Commands,
    replicon_channels       : Res<RepliconChannels>,
){
    // get server/client channels
    let server_channels = replicon_channels.get_server_configs();
    let client_channels = replicon_channels.get_client_configs();

    // make server
    let (client, client_transport) = create_wasm_wt_client(
        server_channels.clone(),
        client_channels.clone(),
        authentication,
        config,
    );

    // add client and transport
    client_app_commands.insert_resource(client);
    client_app_commands.insert_resource(client_transport);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Sets up a renet client with wasm websocket transport using the provided authentication and client address.
/// - Assumes there is a `bevy_replicon::RepliconChannels` resource already loaded in the app.
#[cfg(target_family = "wasm")]
fn setup_wasm_ws_renet_client(
    In((
        authentication,
        config
    ))                      : In<(ClientAuthentication, bevy_renet2::netcode::WebSocketClientConfig)>,
    mut client_app_commands : Commands,
    replicon_channels       : Res<RepliconChannels>,
){
    // get server/client channels
    let server_channels = replicon_channels.get_server_configs();
    let client_channels = replicon_channels.get_client_configs();

    // make server
    let (client, client_transport) = create_wasm_ws_client(
        server_channels.clone(),
        client_channels.clone(),
        authentication,
        config,
    );

    // add client and transport
    client_app_commands.insert_resource(client);
    client_app_commands.insert_resource(client_transport);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Sets up a renet client with in-memory transport using the provided authentication and client socket.
/// - Assumes there is a `bevy_replicon::RepliconChannels` resource already loaded in the app.
#[cfg(feature = "memory_transport")]
fn setup_memory_renet_client(
    In((
        authentication,
        client
    ))                      : In<(ClientAuthentication, bevy_renet2::netcode::MemorySocketClient)>,
    mut client_app_commands : Commands,
    replicon_channels       : Res<RepliconChannels>,
){
    // get server/client channels
    let server_channels = replicon_channels.get_server_configs();
    let client_channels = replicon_channels.get_client_configs();

    // make server
    let (client, client_transport) = create_memory_client(
        server_channels.clone(),
        client_channels.clone(),
        authentication,
        client,
    );

    // add client and transport
    client_app_commands.insert_resource(client);
    client_app_commands.insert_resource(client_transport);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Sets up a renet client with pre-loaded connection information.
/// - Removes the [`ClientConnectPack`] resource from the world, or returns an error if it is missing.
pub fn setup_renet_client(world: &mut World) -> Result<(), ()>
{
    tracing::info!("setting up renet client");

    let connect_pack = world.remove_resource::<ClientConnectPack>().ok_or(())?;

    // drop the existing transport to free its address(es) in case we are re-using a client address
    // - Note that this doesn't guarantee all addresses are freed, as some may not be freed until an async shutdown
    //   procedure is completed.
    world.remove_resource::<NetcodeClientTransport>();

    match connect_pack
    {
        #[cfg(feature = "memory_transport")]
        ClientConnectPack::Memory(authentication, client) =>
        {
            world.syscall((authentication, client), setup_memory_renet_client);
        }
        ClientConnectPack::Native(authentication, client_address) =>
        {
            world.syscall((authentication, client_address), setup_native_renet_client);
        }
        #[cfg(target_family = "wasm")]
        ClientConnectPack::WasmWt(authentication, config) =>
        {
            world.syscall((authentication, config), setup_wasm_wt_renet_client);
        }
        #[cfg(target_family = "wasm")]
        ClientConnectPack::WasmWs(authentication, config) =>
        {
            world.syscall((authentication, config), setup_wasm_ws_renet_client);
        }
    }

    Ok(())
}

//-------------------------------------------------------------------------------------------------------------------
