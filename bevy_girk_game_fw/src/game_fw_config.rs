//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Game framework config
#[derive(Resource, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GameFwConfig
{
    /// Tick rate of the game per second.
    ///
    /// Must be at least 1.
    ///
    /// This should be used to set the game app's update rate.
    //todo: need to use FixedUpdate instead? the goal is tick-oriented progress accounting for easy determinism; must
    //      maintain synchronization between server messages and server connection events (which may be lost due to
    //      bevy Event handling)
    //      - maybe split between full-update ticks and inner fixed-update ticks
    ticks_per_sec: u32,
    /// Maximum number of ticks that may elapse in game framework initialization.
    ///
    /// Must be at least 1.
    ///
    /// [`GameFwState::Init`] will always end when these ticks have elapsed even if clients are not ready.
    max_init_ticks: u32,
    /// Maximum number of ticks that may elapse after game over before the app exits.
    ///
    /// This is included because not exiting immediately allows time to propagate the game end state change to clients,
    /// and to allow custom app termination in game logic (i.e. by setting the max end ticks to infinite).
    max_end_ticks: u32,
}

impl GameFwConfig
{
    /// New game framework config
    pub fn new(
        ticks_per_sec  : u32,
        max_init_ticks : u32,
        max_end_ticks  : u32,
    ) -> GameFwConfig 
    {
        if ticks_per_sec == 0 { panic!("tick rate must be > 0!"); }
        if max_init_ticks == 0 { panic!("max init ticks must be > 0!"); }
        GameFwConfig{
                ticks_per_sec,
                max_init_ticks,
                max_end_ticks,
            }
    }

    /// Gets the tick rate of the game.
    pub fn ticks_per_sec(&self) -> u32 { self.ticks_per_sec }

    /// Gets the maximum number of game-init ticks.
    pub fn max_init_ticks(&self) -> u32 { self.max_init_ticks }

    /// Gets the maximum number of game-end ticks.
    pub fn max_end_ticks(&self) -> u32 { self.max_end_ticks }
}

//-------------------------------------------------------------------------------------------------------------------
