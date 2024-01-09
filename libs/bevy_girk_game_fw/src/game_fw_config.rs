//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Game framework config
#[derive(Resource, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GameFwConfig
{
    /// Tick rate of the game.
    ticks_per_sec: Ticks,
    /// Maximum number of ticks that may elapse in game framework initialization.
    max_init_ticks: Ticks,
    /// Maximum number of ticks that may elapse after game over.
    max_end_ticks: Ticks,
}

impl GameFwConfig
{
    /// New game framework config
    pub fn new(
        ticks_per_sec  : Ticks,
        max_init_ticks : Ticks,
        max_end_ticks  : Ticks,
    ) -> GameFwConfig 
    {
        if ticks_per_sec == Ticks(0) { panic!("GameFwConfig: tick rate must be > 0!"); }
        GameFwConfig{
                ticks_per_sec,
                max_init_ticks,
                max_end_ticks,
            }
    }

    pub fn ticks_per_sec(&self) -> Ticks { self.ticks_per_sec }
    pub fn max_init_ticks(&self) -> Ticks { self.max_init_ticks }
    pub fn max_end_ticks(&self) -> Ticks { self.max_end_ticks }
}

//-------------------------------------------------------------------------------------------------------------------
