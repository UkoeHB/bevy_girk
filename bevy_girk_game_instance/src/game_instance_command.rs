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
    ///
    /// Causes the game to exit with an error code:
    /// - `65`: Indicates the app was unable to forward a [`GameInstanceReport::Aborted`] report to the owner.
    /// - `66`: Indicates the app was successfully aborted.
    Abort
}

//-------------------------------------------------------------------------------------------------------------------
