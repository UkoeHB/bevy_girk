//local shortcuts

//third-party shortcuts
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Report emitted by a client instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientInstanceReport
{
    /// Request a new connect token.
    RequestConnectToken,
    /// The client instance was aborted.
    Aborted(u64),
}

//-------------------------------------------------------------------------------------------------------------------
