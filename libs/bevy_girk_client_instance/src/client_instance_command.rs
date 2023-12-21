//local shortcuts

//third-party shortcuts
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientInstanceCommand
{
    /// Abort the client instance.
    Abort,
    /// Reconnect to the server with a fresh connect token.
    Connect(ServerConnectToken)
}

//-------------------------------------------------------------------------------------------------------------------
