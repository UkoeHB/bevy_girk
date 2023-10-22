//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Game framework mode
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum GameFWMode
{
    #[default]
    Init,
    Game,
    End
}

//-------------------------------------------------------------------------------------------------------------------
