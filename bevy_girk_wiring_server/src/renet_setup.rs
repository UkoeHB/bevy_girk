//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy_girk_wiring_common::{ConnectMetaMemory, ConnectMetaNative, ConnectMetaWasm, GameServerSetupConfig};
use bevy_replicon::core::channels::RepliconChannels;
use bevy_replicon_renet2::RenetChannelsExt;
use renet2::{ChannelConfig, ConnectionConfig, RenetServer};
use renet2::transport::{
    in_memory_server_addr, new_memory_sockets, BoxedSocket, NetcodeServerTransport, ServerAuthentication,
    ServerSetupConfig, TransportSocket
};

//standard shortcuts
use std::net::SocketAddr;
use wasm_timer::{SystemTime, UNIX_EPOCH};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_memory_socket(
    config: &GameServerSetupConfig,
    memory_clients: Vec<u16>,
    socket_addresses: &mut Vec<Vec<SocketAddr>>,
    sockets: &mut Vec<BoxedSocket>,
    auth_key: &[u8; 32],
) -> Option<ConnectMetaMemory>
{
    if memory_clients.len() == 0 { return None }

    #[cfg(not(feature = "memory_transport"))]
    {
        panic!("tried setting up renet server with in-memory clients, but memory_transport feature is not enabled");
    }

    #[cfg(feature = "memory_transport")]
    {
        let (server_socket, client_sockets) = new_memory_sockets(memory_clients, true);
        let addrs = vec![in_memory_server_addr()];

        let meta = ConnectMetaMemory {
            server_config: config.clone(),
            clients: client_sockets,
            socket_id: sockets.len() as u8,  // DO THIS BEFORE PUSHING SOCKET
            auth_key: auth_key.clone(),
        };

        socket_addresses.push(addrs);
        sockets.push(BoxedSocket::new(server_socket));

        Some(meta)
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_native_socket(
    config: &GameServerSetupConfig,
    native_count: usize,
    socket_addresses: &mut Vec<Vec<SocketAddr>>,
    sockets: &mut Vec<BoxedSocket>,
    auth_key: &[u8; 32],
) -> Option<ConnectMetaNative>
{
    if native_count == 0 { return None }

    #[cfg(not(feature = "native_transport"))]
    {
        panic!("tried setting up renet server with native clients, but native_transport feature is not enabled");
    }

    #[cfg(feature = "native_transport")]
    {
        let wildcard_addr = SocketAddr::new(config.server_ip.into(), 0);
        let server_socket = std::net::UdpSocket::bind(wildcard_addr).expect("renet server address should be bindable");
        let socket = renet2::transport::NativeSocket::new(server_socket).unwrap();
        let addrs = vec![socket.addr().unwrap()];

        let meta = ConnectMetaNative {
            server_config: config.clone(),
            server_addresses: addrs.clone(),
            socket_id: sockets.len() as u8,  // DO THIS BEFORE PUSHING SOCKET
            auth_key: auth_key.clone(),
        };

        socket_addresses.push(addrs);
        sockets.push(BoxedSocket::new(socket));

        Some(meta)
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_wasm_socket(
    config: &GameServerSetupConfig,
    wasm_count: usize,
    socket_addresses: &mut Vec<Vec<SocketAddr>>,
    sockets: &mut Vec<BoxedSocket>,
    auth_key: &[u8; 32],
) -> Option<ConnectMetaWasm>
{
    if wasm_count == 0 { return None }

    #[cfg(not(feature = "wasm_transport"))]
    {
        panic!("tried setting up renet server with wasm clients, but wasm_transport feature is not enabled");
    }

    #[cfg(feature = "wasm_transport")]
    {
        use enfync::AdoptOrDefault;
        let wildcard_addr = SocketAddr::new(config.server_ip.into(), 0);
        let (wt_config, cert_hash) = renet2::transport::WebTransportServerConfig::new_selfsigned(wildcard_addr, wasm_count);
        let handle = enfync::builtin::native::TokioHandle::adopt_or_default();  //todo: don't depend on tokio...
        let socket = renet2::transport::WebTransportServer::new(wt_config, handle.0).unwrap();
        let addrs = vec![socket.addr().unwrap()];

        let meta = ConnectMetaWasm {
            server_config: config.clone(),
            server_addresses: addrs.clone(),
            socket_id: sockets.len() as u8,  // DO THIS BEFORE PUSHING SOCKET
            auth_key: auth_key.clone(),
            cert_hashes: vec![cert_hash],
        };

        socket_addresses.push(addrs);
        sockets.push(BoxedSocket::new(socket));

        Some(meta)
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "native_transport")]
pub(crate) fn create_native_server(
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
    let server_socket = std::net::UdpSocket::bind(server_config.socket_addresses[0][0])
        .expect("renet server address should be bindable");
    server_config.socket_addresses = vec![vec![server_socket.local_addr().unwrap()]];

    // make transport
    let server_transport = NetcodeServerTransport::new(server_config, renet2::transport::NativeSocket::new(server_socket).unwrap()).unwrap();

    (server, server_transport)
}

//-------------------------------------------------------------------------------------------------------------------

/// Set up a renet server with default transport using the provided `ServerConfig`.
/// - Assumes there is a bevy_replicon::RepliconChannels resource already loaded in the app.
#[cfg(feature = "native_transport")]
pub fn setup_native_renet_server(server_app: &mut App, server_config: ServerSetupConfig) -> SocketAddr
{
    tracing::info!("setting up renet server");

    // get server/client channels
    let replicon_channels = server_app.world().resource::<RepliconChannels>();
    let server_channels   = replicon_channels.get_server_configs();
    let client_channels   = replicon_channels.get_client_configs();

    // make server
    let (server, server_transport) = create_native_server(
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

/// Set up a renet server with arbitrary combinations of memory/native/wasm transports.
/// - Assumes there is a bevy_replicon::RepliconChannels resource already loaded in the app.
pub fn setup_combo_renet_server(
    server_app: &mut App,
    config: GameServerSetupConfig,
    memory_clients: Vec<u16>,
    native_count: usize,
    wasm_count: usize,
    auth_key: &[u8; 32],
) -> (Option<ConnectMetaMemory>, Option<ConnectMetaNative>, Option<ConnectMetaWasm>)
{
    tracing::info!("setting up renet server");

    let memory_count = memory_clients.len();

    // get server/client channels
    let replicon_channels = server_app.world().resource::<RepliconChannels>();
    let server_channels_config = replicon_channels.get_server_configs();
    let client_channels_config = replicon_channels.get_client_configs();

    let server = RenetServer::new(
        ConnectionConfig{
            server_channels_config,
            client_channels_config,
            ..default()
        }
    );

    // add sockets
    let mut socket_addresses = Vec::default();
    let mut sockets = Vec::default();

    let memory_meta = add_memory_socket(&config, memory_clients, &mut socket_addresses, &mut sockets, auth_key);
    let native_meta = add_native_socket(&config, native_count, &mut socket_addresses, &mut sockets, auth_key);
    let wasm_meta = add_wasm_socket(&config, wasm_count, &mut socket_addresses, &mut sockets, auth_key);

    // save final addresses
    let server_config = ServerSetupConfig {
        current_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default(),
        max_clients: memory_count + native_count + wasm_count,
        protocol_id: config.protocol_id,
        socket_addresses,
        authentication: ServerAuthentication::Secure{ private_key: *auth_key },
    };

    // make transport
    let server_transport = NetcodeServerTransport::new_with_sockets(server_config, sockets).unwrap();

    // add server to app
    server_app
        .insert_resource(server)
        .insert_resource(server_transport);

    (memory_meta, native_meta, wasm_meta)
}

//-------------------------------------------------------------------------------------------------------------------
