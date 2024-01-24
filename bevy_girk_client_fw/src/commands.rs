//local shortcuts

//third-party shortcuts
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Commands that may be sent to the client by the client framework's controller.
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientFwCommand
{
    /// Reinitialize the client.
    ReInitialize,
    /// Filler variant.
    None
}

//-------------------------------------------------------------------------------------------------------------------
