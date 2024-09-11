//local shortcuts

//third-party shortcuts
use bevy_renet2::renet2::transport::{ConnectToken, ServerCertHash};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct TargetClient(pub u64);
#[derive(Debug, Copy, Clone)]
pub struct TargetAll;
#[derive(Debug, Copy, Clone)]
pub struct TargetAllExcept(pub u64);

impl From<TargetClient> for SendMode
{
    fn from(client: TargetClient) -> SendMode { return SendMode::Direct(ClientId::new(client.0)) }
}
impl From<TargetAll> for SendMode
{
    fn from(_: TargetAll) -> SendMode { return SendMode::Broadcast }
}
impl From<TargetAllExcept> for SendMode
{
    fn from(exception: TargetAllExcept) -> SendMode { return SendMode::BroadcastExcept(ClientId::new(exception.0)) }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct SendUnreliable;
#[derive(Debug, Copy, Clone)]
pub struct SendUnordered;
#[derive(Debug, Copy, Clone)]
pub struct SendOrdered;

impl From<SendUnreliable> for ChannelKind
{
    fn from(_: SendUnreliable) -> ChannelKind { return ChannelKind::Unreliable }
}
impl From<SendUnordered> for ChannelKind
{
    fn from(_: SendUnordered) -> ChannelKind { return ChannelKind::Unordered }
}
impl From<SendOrdered> for ChannelKind
{
    fn from(_: SendOrdered) -> ChannelKind { return ChannelKind::Ordered }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for converting a message type into its send policy.
///
/// Especially useful for enum-type messages where different variants have different send policies.
pub trait IntoChannelKind
{
    fn into_event_type(&self) -> ChannelKind;
}

//-------------------------------------------------------------------------------------------------------------------

/// Default implementation for tests.
impl IntoChannelKind for ()
{
    fn into_event_type(&self) -> ChannelKind { ChannelKind::Unreliable }
}

//-------------------------------------------------------------------------------------------------------------------

/// Configuration details for setting up a renet server.
/// - Used to set up renet servers for clients on native targets.
//todo: include setup configs for wasm and in-memory transports (each one optional?)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameServerSetupConfig
{
    /// Protocol id for server/client compatibility.
    pub protocol_id: u64,
    /// How long connect tokens should live before expiring.
    pub expire_secs: u64,
    /// Internal connection timeout for clients and servers.
    pub timeout_secs: i32,
    /// The server's IP address.
    ///
    /// The server port will be auto-selected.
    pub server_ip: Ipv6Addr,
}

impl GameServerSetupConfig
{
    /// Make a dummy config.
    /// 
    /// Should not be used to connect to a real renet server.
    pub fn dummy() -> Self
    {
        Self {
            protocol_id: 0u64,
            expire_secs: 10u64,
            timeout_secs: 5i32,
            server_ip: Ipv6Addr::LOCALHOST,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Metadata required to generate connect tokens for in-memory clients.
#[cfg(feature = "memory_transport")]
#[derive(Debug, Clone)]
pub struct ConnectMetaMemory
{
    pub server_config: GameServerSetupConfig,
    pub clients: Vec<MemorySocketClient>,
    pub socket_id: u8,
    pub auth_key: [u8; 32],
}

#[cfg(not(feature = "memory_transport"))]
#[derive(Debug, Clone)]
pub struct ConnectMetaMemory;

//-------------------------------------------------------------------------------------------------------------------

/// Metadata required to generate connect tokens for native-target clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectMetaNative
{
    pub server_config: GameServerSetupConfig,
    pub server_addresses: Vec<SocketAddr>,
    pub socket_id: u8,
    pub auth_key: [u8; 32],
}

impl ConnectMetaNative
{
    pub fn dummy() -> Self
    {
        let mut auth_key = [0u8; 32];
        auth_key[0] = 1;

        Self{
            server_config: GameServerSetupConfig::dummy(),
            server_addresses: vec![SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080u16))],
            socket_id: 0,
            auth_key,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Metadata required to generate connect tokens for wasm-target clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectMetaWasm
{
    pub server_config    : GameServerSetupConfig,
    pub server_addresses : Vec<SocketAddr>,
    pub socket_id        : u8,
    pub auth_key         : [u8; 32],
    pub cert_hashes      : Vec<ServerCertHash>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Generates a new connect token for an in-memory client.
///
/// Note that [`ConnectMetaMemory`] can contain sockets for multiple clients. You must specify which client from
/// that list is needed here with the `client_index` parameter.
#[cfg(feature = "memory_transport")]
pub fn new_connect_token_memory(
    meta         : &ConnectMetaMemory,
    client_index : usize,
    current_time : Duration,
    client_id    : u64,
) -> Option<ServerConnectToken>
{
    let token = ConnectToken::generate(
        current_time,
        meta.server_config.protocol_id,
        meta.server_config.expire_secs,
        client_id,
        meta.server_config.timeout_secs,
        meta.socket_id,
        vec![in_memory_server_addr()],
        None,
        &meta.auth_key,
    ).ok()?;

    Some(ServerConnectToken::Memory{
        token: connect_token_to_bytes(&token)?,
        client: meta.clients.get(client_index)?.clone()
    })
}

//-------------------------------------------------------------------------------------------------------------------

/// Generates a new connect token for a native client.
pub fn new_connect_token_native(
    meta         : &ConnectMetaNative,
    current_time : Duration,
    client_id    : u64,
) -> Option<ServerConnectToken>
{
    let token = ConnectToken::generate(
        current_time,
        meta.server_config.protocol_id,
        meta.server_config.expire_secs,
        client_id,
        meta.server_config.timeout_secs,
        meta.socket_id,
        meta.server_addresses.clone(),
        None,
        &meta.auth_key,
    ).ok()?;

    Some(ServerConnectToken::Native{ token: connect_token_to_bytes(&token)? })
}

//-------------------------------------------------------------------------------------------------------------------

/// Generates a new connect token for a wasm client.
pub fn new_connect_token_wasm(
    meta         : &ConnectMetaWasm,
    current_time : Duration,
    client_id    : u64,
) -> Option<ServerConnectToken>
{
    let token = ConnectToken::generate(
        current_time,
        meta.server_config.protocol_id,
        meta.server_config.expire_secs,
        client_id,
        meta.server_config.timeout_secs,
        meta.socket_id,
        meta.server_addresses.clone(),
        None,
        &meta.auth_key,
    ).ok()?;

    Some(ServerConnectToken::Wasm{ token: connect_token_to_bytes(&token)?, cert_hashes: meta.cert_hashes.clone() })
}

//-------------------------------------------------------------------------------------------------------------------

pub fn connect_token_to_bytes(connect_token: &ConnectToken) -> Option<Vec<u8>>
{
    let mut bytes = Vec::<u8>::with_capacity(std::mem::size_of::<ConnectToken>());
    connect_token.write(&mut bytes).ok()?;
    Some(bytes)
}

//-------------------------------------------------------------------------------------------------------------------

pub fn connect_token_from_bytes(connect_token_bytes: &Vec<u8>) -> Option<ConnectToken>
{
    ConnectToken::read(&mut &connect_token_bytes[..]).ok()
}

//-------------------------------------------------------------------------------------------------------------------

/// A token that a client can use to connect to a renet server.
//todo: how to serialize the connect token more directly to reduce allocations?
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerConnectToken
{
    Native{
        /// A renet `ConnectToken`.
        #[serde_as(as = "Bytes")]
        token: Vec<u8>
    },
    //todo: consider making this more flexible in case you don't want the cert hash workflow
    Wasm{
        /// A renet [`ConnectToken`].
        #[serde_as(as = "Bytes")]
        token: Vec<u8>,
        /// Cert hashes for connecting to self-signed servers.
        cert_hashes: Vec<ServerCertHash>
    },
    #[cfg(feature = "memory_transport")]
    #[serde(skip)]
    Memory{
        /// A renet [`ConnectToken`].
        token: Vec<u8>,
        /// In-memory channel the client will use to talk to the renet server.
        client: MemorySocketClient,
    }
}

impl Default for ServerConnectToken
{
    fn default() -> Self { Self::Native{ token: vec![] } }
}

//-------------------------------------------------------------------------------------------------------------------

/// Get an unspecified client address from a server address.
///
/// The type of the client address returned will be tailored to the type of the server address (Ipv4/Ipv6).
pub fn client_address_from_server_address(server_addr: &SocketAddr) -> SocketAddr
{
    match server_addr
    {
        SocketAddr::V4(_) => SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
        SocketAddr::V6(_) => SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0),
    }
}

//-------------------------------------------------------------------------------------------------------------------
