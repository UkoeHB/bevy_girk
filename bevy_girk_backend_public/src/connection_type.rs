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
    /// Use this when the client has [`bevy_simplenet::EnvType::Wasm`].
    Wasm,
}

impl From<bevy_simplenet::EnvType> for ConnectionType
{
    fn from(env: bevy_simplenet::EnvType) -> Self
    {
        match env
        {
            bevy_simplenet::EnvType::Native => Self::Native,
            bevy_simplenet::EnvType::Wasm => Self::Wasm,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
