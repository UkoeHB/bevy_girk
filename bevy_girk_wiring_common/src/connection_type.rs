use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

/// Represents the type of connection a client wants to make with game servers.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ConnectionType
{
    /// Use this when the client and server are in the same binary (e.g. local-player).
    Memory,
    /// Use this when the client is non-WASM.
    Native,
    /// Use this when the client is WASM and webtransport with cert hashes is supported.
    WasmWt,
    /// Use this when the client is WASM and webtransport is not supported.
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

        #[cfg(all(target_family = "wasm", feature = "wasm_transport_wt"))]
        {
            match renet2_netcode::webtransport_is_available_with_cert_hashes() {
                true => ConnectionType::WasmWt,
                false => ConnectionType::WasmWs,
            }
        }

        #[cfg(all(target_family = "wasm", not(feature = "wasm_transport_wt")))]
        {
            ConnectionType::WasmWs
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
