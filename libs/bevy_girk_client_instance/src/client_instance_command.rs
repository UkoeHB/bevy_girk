//local shortcuts
use bevy_girk_utils::*;

//third-party shortcuts
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// A command that may be sent into a client instance.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientInstanceCommand
{
    /// Abort the client instance.
    Abort,
    /// Reconnect to the server with a fresh connect token.
    Connect(ServerConnectToken)
}

//-------------------------------------------------------------------------------------------------------------------
