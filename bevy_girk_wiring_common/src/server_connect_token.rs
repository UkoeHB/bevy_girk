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
