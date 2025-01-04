//local shortcuts

//third-party shortcuts
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Represents the type of connection a client wants to make with game servers.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ConnectionType
{
    /// Use this when the client and server are in the same binary (e.g. local-player).
    Memory,
    /// Use this when the client has [`bevy_simplenet::EnvType::Native`].
    Native,
    /// Use this when the client has [`bevy_simplenet::EnvType::Wasm`] and webtransport with cert hashes is
    /// supported.
    WasmWt,
    /// Use this when the client has [`bevy_simplenet::EnvType::Wasm`] and webtransport is not supported.
    WasmWs,
}

impl ConnectionType
{
    /// Infers the connection type from the environment.
    pub fn inferred() -> Self
    {
        #[cfg(not(target_family = "wasm"))]
        {
            ConnectionType::Native
        }

        #[cfg(target_family = "wasm")]
        {
            match renet2_netcode::webtransport_is_available_with_cert_hashes() {
                true => ConnectionType::WasmWt,
                false => ConnectionType::WasmWs,
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
