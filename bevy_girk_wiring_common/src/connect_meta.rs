//local shortcuts
use crate::*;

//third-party shortcuts
use renet2_netcode::{in_memory_server_addr, ConnectToken, MemorySocketClient, ServerCertHash};
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::{net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4}, time::Duration};

//-------------------------------------------------------------------------------------------------------------------

/// Configuration details for setting up a renet server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameServerSetupConfig
{
    /// Protocol id for server/client compatibility.
    pub protocol_id: u64,
    /// How long connect tokens should live before expiring.
    pub expire_secs: u64,
    /// Internal connection timeout for clients and servers.
    pub timeout_secs: i32,
    /// The server's IP address. Used for both native and WASM server sockets.
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

#[cfg(feature = "memory_transport")]
impl ConnectMetaMemory
{
    /// Generates a new connect token for an in-memory client.
    ///
    /// Note that [`ConnectMetaMemory`] can contain sockets for multiple clients. We search available clients for
    /// the requested client id, and return `None` on failure.
    pub fn new_connect_token(
        &self,
        current_time : Duration,
        client_id    : u64,
    ) -> Option<ServerConnectToken>
    {
        let token = ConnectToken::generate(
            current_time,
            self.server_config.protocol_id,
            self.server_config.expire_secs,
            client_id,
            self.server_config.timeout_secs,
            self.socket_id,
            vec![in_memory_server_addr()],
            None,
            &self.auth_key,
        ).ok()?;
        let token = connect_token_to_bytes(&token)?;
        let client = self.clients.iter().find(|c| c.id() == client_id).cloned()?;

        Some(ServerConnectToken::Memory{ token, client })
    }
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

    /// Generates a new connect token for a native client.
    pub fn new_connect_token(
        &self,
        current_time : Duration,
        client_id    : u64,
    ) -> Option<ServerConnectToken>
    {
        let token = ConnectToken::generate(
            current_time,
            self.server_config.protocol_id,
            self.server_config.expire_secs,
            client_id,
            self.server_config.timeout_secs,
            self.socket_id,
            self.server_addresses.clone(),
            None,
            &self.auth_key,
        ).ok()?;

        Some(ServerConnectToken::Native{ token: connect_token_to_bytes(&token)? })
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Metadata required to generate connect tokens for wasm-target clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectMetaWasmWt
{
    pub server_config    : GameServerSetupConfig,
    pub server_addresses : Vec<SocketAddr>,
    pub socket_id        : u8,
    pub auth_key         : [u8; 32],
    pub cert_hashes      : Vec<ServerCertHash>,
}

impl ConnectMetaWasmWt
{
    /// Generates a new connect token for a wasm client.
    pub fn new_connect_token(
        &self,
        current_time : Duration,
        client_id    : u64,
    ) -> Option<ServerConnectToken>
    {
        let token = ConnectToken::generate(
            current_time,
            self.server_config.protocol_id,
            self.server_config.expire_secs,
            client_id,
            self.server_config.timeout_secs,
            self.socket_id,
            self.server_addresses.clone(),
            None,
            &self.auth_key,
        ).ok()?;

        Some(ServerConnectToken::WasmWt{
            token: connect_token_to_bytes(&token)?,
            cert_hashes: self.cert_hashes.clone()
        })
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Metadata required to generate connect tokens for wasm-target clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectMetaWasmWs
{
    pub server_config    : GameServerSetupConfig,
    pub server_addresses : Vec<SocketAddr>,
    pub socket_id        : u8,
    pub auth_key         : [u8; 32],
    pub url              : url::Url,
}

impl ConnectMetaWasmWs
{
    /// Generates a new connect token for a wasm client.
    pub fn new_connect_token(
        &self,
        current_time : Duration,
        client_id    : u64,
    ) -> Option<ServerConnectToken>
    {
        let token = ConnectToken::generate(
            current_time,
            self.server_config.protocol_id,
            self.server_config.expire_secs,
            client_id,
            self.server_config.timeout_secs,
            self.socket_id,
            self.server_addresses.clone(),
            None,
            &self.auth_key,
        ).ok()?;

        Some(ServerConnectToken::WasmWs{
            token: connect_token_to_bytes(&token)?,
            url: self.url.clone()
        })
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Metadata required to generate connect tokens for girk clients.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConnectMetas
{
    #[serde(skip)]
    pub memory: Option<ConnectMetaMemory>,
    pub native: Option<ConnectMetaNative>,
    pub wasm_wt: Option<ConnectMetaWasmWt>,
    pub wasm_ws: Option<ConnectMetaWasmWs>
}

//-------------------------------------------------------------------------------------------------------------------
