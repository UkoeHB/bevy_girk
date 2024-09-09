//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// The game framework state.
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum GameFwState
{
    /// The game is initializing and at least one client is not ready.
    #[default]
    Init,
    /// The game is running.
    Game,
    /// The game is over (a game over report was emitted) and the framework is waiting for the app to close.
    End
}

//-------------------------------------------------------------------------------------------------------------------
