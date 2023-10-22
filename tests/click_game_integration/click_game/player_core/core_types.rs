//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Client core mode
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ClientCoreMode
{
    #[default]
    Init,
    Prep,
    Play,
    GameOver
}

//-------------------------------------------------------------------------------------------------------------------
