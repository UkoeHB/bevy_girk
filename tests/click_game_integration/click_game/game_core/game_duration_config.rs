//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Configuration details for game duration.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct GameDurationConfig
{
    /// Number of ticks that should elapse in [GameState::Play] before switching [GameState::Play] -> [GameState::GameOver].
    game_ticks: u32,
    // The first 'game over' tick will occur after 'prep_ticks + game_ticks' ticks have elapsed.
}

impl GameDurationConfig
{
    pub fn new(game_ticks : u32) -> Self
    {
        Self{ game_ticks }
    }

    pub fn expected_state(&self, game_tick: Tick) -> GameState
    {
        // play
        if *game_tick < self.game_ticks { return GameState::Play; }

        // game over
        GameState::GameOver
    }
}

//-------------------------------------------------------------------------------------------------------------------
