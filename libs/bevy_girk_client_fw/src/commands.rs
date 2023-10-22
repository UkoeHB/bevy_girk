//local shortcuts

//third-party shortcuts
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Commands that may come from the client framework's controller.
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientFWCommand
{
    /// Reinitialize the client.
    ReInitialize,
    /// Filler variant.
    None
}

//-------------------------------------------------------------------------------------------------------------------
