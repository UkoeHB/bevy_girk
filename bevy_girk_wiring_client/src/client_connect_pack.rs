//local shortcuts
use bevy_girk_wiring_common::{
    client_address_from_server_address, connect_token_from_bytes, ServerConnectToken
};

//third-party shortcuts
use bevy::prelude::*;
use renet2::transport::ClientAuthentication;

//standard shortcuts
use std::net::SocketAddr;

//-------------------------------------------------------------------------------------------------------------------

/// Information needed to connect a renet client to a renet server.
///
/// Add this as a resource to your app before trying to set up a renet client.
///
/// Connect packs should be considered single-use. If you need to reconnect, make a new connect pack with fresh
/// client authentication.
#[derive(Resource, Debug)]
pub enum ClientConnectPack
{
    /// Connection information for native transports.
    /// 
    /// Note: The client address should be tailored to the server address type (Ipv4/Ipv6).
    Native(ClientAuthentication, SocketAddr),
    /// Connection information for wasm transports.
    #[cfg(target_family = "wasm")]
    Wasm(ClientAuthentication, renet2::transport::WebTransportClientConfig),
    #[cfg(feature = "memory_transport")]
    Memory(ClientAuthentication, renet2::transport::MemorySocketClient),
}

impl ClientConnectPack
{
    /// Make a new connect pack from a server connect token.
    pub fn new(expected_protocol_id: u64, token: ServerConnectToken) -> Result<Self, String>
    {
        match token
        {
            ServerConnectToken::Native{ token } =>
            {
                // Extract renet ConnectToken.
                let connect_token = connect_token_from_bytes(&token)
                    .ok_or(String::from("failed deserializing renet connect token"))?;
                if connect_token.protocol_id != expected_protocol_id { return Err(String::from("protocol id mismatch")); }

                // prepare client address based on server address
                let Some(server_addr) = connect_token.server_addresses[0]
                else { return Err(String::from("server address is missing")); };
                let client_address = client_address_from_server_address(&server_addr);

                Ok(Self::Native(ClientAuthentication::Secure{ connect_token }, client_address))
            }
            ServerConnectToken::Wasm{ token, cert_hashes } =>
            {
                #[cfg(target_family = "wasm")]
                {
                    // Extract renet ConnectToken.
                    let connect_token = connect_token_from_bytes(&token)
                        .ok_or(String::from("failed deserializing renet connect token"))?;
                    if connect_token.protocol_id != expected_protocol_id { return Err(String::from("protocol id mismatch")); }

                    // prepare client config based on server address
                    let Some(server_addr) = connect_token.server_addresses[0]
                    else { return Err(String::from("server address is missing")); };
                    let config = renet2::transport::WebTransportClientConfig::new_with_certs(server_addr, cert_hashes);

                    Ok(Self::Wasm(ClientAuthentication::Secure{ connect_token }, config))
                }
                #[cfg(not(target_family = "wasm"))]
                {
                    let (_, _) = (token, cert_hashes);
                    panic!("ServerConnectToken::Wasm can only be converted to ClientConnectPack in WASM");
                }
            }
            #[cfg(feature = "memory_transport")]
            ServerConnectToken::Memory{ token, client } => 
            {
                // Extract renet ConnectToken.
                let connect_token = connect_token_from_bytes(&token)
                    .ok_or(String::from("failed deserializing renet connect token"))?;
                if connect_token.protocol_id != expected_protocol_id { return Err(String::from("protocol id mismatch")); }

                Ok(Self::Memory(ClientAuthentication::Secure{ connect_token }, client))
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------