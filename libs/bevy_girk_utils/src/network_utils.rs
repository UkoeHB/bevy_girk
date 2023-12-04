//local shortcuts

//third-party shortcuts
use bevy_renet::renet::ClientId;
use bevy_renet::renet::transport::ConnectToken;
use bevy_replicon::prelude::*;
use bevy_replicon::network_event::EventType;
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
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

#[derive(Serialize, Deserialize)]
pub struct GameServerSetupConfig
{
    pub protocol_id     : u64,
    pub expire_seconds  : u64,
    pub timeout_seconds : i32,
    pub server_ip       : Ipv6Addr,
}

//-------------------------------------------------------------------------------------------------------------------

pub fn new_connect_token(
    current_time     : Duration,
    server_config    : &GameServerSetupConfig,
    auth_key         : &[u8; 32],
    client_id        : u64,
    server_addresses : Vec<SocketAddr>
) -> ConnectToken
{
    ConnectToken::generate(
            current_time,
            server_config.protocol_id,
            server_config.expire_seconds,
            client_id,
            server_config.timeout_seconds,
            server_addresses,
            None,
            auth_key,
        ).expect("failed to make connect token")
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

pub fn connect_token_from_bytes(connect_token_bytes: Vec<u8>) -> Result<ConnectToken, ()>
{
    ConnectToken::read(&mut &connect_token_bytes[..]).map_err(|_| ())
}

//-------------------------------------------------------------------------------------------------------------------

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
    //The server app will contain `Res<[client transports]>` which you must extract and insert to your client apps
    //manually.
    //InMemory,
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
