//local shortcuts

//third-party shortcuts
use bevy_renet::renet::ClientId;
use bevy_renet::renet::transport::ConnectToken;
use bevy_replicon::prelude::*;
use bevy_replicon::network_event::EventType;
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
    fn from(client: TargetClient) -> SendMode { return SendMode::Direct(ClientId::from_raw(client.0)) }
}
impl From<TargetAll> for SendMode
{
    fn from(_: TargetAll) -> SendMode { return SendMode::Broadcast }
}
impl From<TargetAllExcept> for SendMode
{
    fn from(exception: TargetAllExcept) -> SendMode { return SendMode::BroadcastExcept(ClientId::from_raw(exception.0)) }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct SendUnreliable;
#[derive(Debug, Copy, Clone)]
pub struct SendUnordered;
#[derive(Debug, Copy, Clone)]
pub struct SendOrdered;

impl From<SendUnreliable> for EventType
{
    fn from(_: SendUnreliable) -> EventType { return EventType::Unreliable }
}
impl From<SendUnordered> for EventType
{
    fn from(_: SendUnordered) -> EventType { return EventType::Unordered }
}
impl From<SendOrdered> for EventType
{
    fn from(_: SendOrdered) -> EventType { return EventType::Ordered }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for converting a message type into its send policy.
///
/// Especially useful for enum-type messages where different variants have different send policies.
pub trait IntoEventType
{
    fn into_event_type(&self) -> EventType;
}

//-------------------------------------------------------------------------------------------------------------------

/// Default implementation for tests.
impl IntoEventType for ()
{
    fn into_event_type(&self) -> EventType { EventType::Unreliable }
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

/// Metadata required to generate connect tokens for native-target clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectMetaNative
{
    pub server_config    : GameServerSetupConfig,
    pub server_addresses : Vec<SocketAddr>,
    pub auth_key         : [u8; 32],
}

impl ConnectMetaNative
{
    pub fn dummy() -> Self
    {
        let mut auth_key = [0u8; 32];
        auth_key[0] = 1;

        Self{
            server_config    : GameServerSetupConfig::dummy(),
            server_addresses : vec![SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080u16))],
            auth_key,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Metadata required to generate connect tokens for wasm-target clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectMetaWasm;

//-------------------------------------------------------------------------------------------------------------------

/// Generate a new connect token for a native client.
pub fn new_connect_token_native(
    meta         : &ConnectMetaNative,
    current_time : Duration,
    client_id    : u64,
) -> Result<ServerConnectToken, ()>
{
    let token =  ConnectToken::generate(
        current_time,
        meta.server_config.protocol_id,
        meta.server_config.expire_secs,
        client_id,
        meta.server_config.timeout_secs,
        meta.server_addresses.clone(),
        None,
        &meta.auth_key,
    ).map_err(|_|())?;

    Ok(ServerConnectToken::Native{ bytes: connect_token_to_bytes(&token)? })
}

//-------------------------------------------------------------------------------------------------------------------

pub fn connect_token_to_bytes(connect_token: &ConnectToken) -> Result<Vec<u8>, ()>
{
    let mut bytes = Vec::<u8>::new();
    bytes.reserve(std::mem::size_of::<ConnectToken>());
    connect_token.write(&mut bytes).map_err(|_| ())?;
    Ok(bytes)
}

//-------------------------------------------------------------------------------------------------------------------

pub fn connect_token_from_bytes(connect_token_bytes: &Vec<u8>) -> Result<ConnectToken, ()>
{
    ConnectToken::read(&mut &connect_token_bytes[..]).map_err(|_| ())
}

//-------------------------------------------------------------------------------------------------------------------

/// A token that a client can use to connect to a renet server.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerConnectToken
{
    /// A renet `ConnectToken` for native transports.
    //todo: how to serialize the connect token more directly to reduce allocations?
    Native{
        #[serde_as(as = "Bytes")]
        bytes: Vec<u8>
    },
    //Wasm(??),
    //InMemory,
    //The server app will contain `Res<[client transports]>` which you must extract and insert to your client apps
    //manually.
}

impl Default for ServerConnectToken
{
    fn default() -> Self { Self::Native{ bytes: vec![] } }
}

//-------------------------------------------------------------------------------------------------------------------

/// Get an unspecified client address from a server address.
///
/// The type of the client address returned will be tailored to the type of the first server address (Ipv4/Ipv6).
pub fn client_address_from_server_address(server_addr: &SocketAddr) -> SocketAddr
{
    match server_addr
    {
        SocketAddr::V4(_) => SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
        SocketAddr::V6(_) => SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0),
    }
}

//-------------------------------------------------------------------------------------------------------------------
