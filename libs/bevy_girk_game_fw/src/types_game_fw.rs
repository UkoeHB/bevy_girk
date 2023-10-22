//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// The number of ticks that have occurred since the game began.
#[derive(Resource, Default)]
pub struct GameFWTicksElapsed
{
    pub elapsed: TicksElapsed
}

//-------------------------------------------------------------------------------------------------------------------

/// Total initialization progress of the game; is replicated to clients.
#[derive(Component, Debug, Default, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct GameInitProgress(pub f32);

#[derive(Bundle, Default)]
pub struct GameInitProgressEntity
{
    progress    : GameInitProgress,
    replication : Replication
}

//-------------------------------------------------------------------------------------------------------------------
