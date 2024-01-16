//local shortcuts

//third-party shortcuts
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// A command that may be sent into a game instance.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameInstanceCommand
{
    /// Abort the instance.
    Abort
}

//-------------------------------------------------------------------------------------------------------------------
