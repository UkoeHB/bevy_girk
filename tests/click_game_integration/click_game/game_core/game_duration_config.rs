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
    /// Number of ticks that should elapse in [GameState::Prep] before switching [GameState::Prep] -> [GameState::Play].
    prep_ticks: u32,
    /// Number of ticks that should elapse in [GameState::Play] before switching [GameState::Play] -> [GameState::GameOver].
    game_ticks: u32,
    // The first 'game over' tick will occur after 'prep_ticks + game_ticks' ticks have elapsed.
}

impl GameDurationConfig
{
    pub fn new(
        prep_ticks : u32,
        game_ticks : u32,
    ) -> GameDurationConfig
    {
        GameDurationConfig{
                prep_ticks,
                game_ticks
            }
    }

    pub fn expected_state(&self, game_tick: Tick) -> GameState
    {
        // prep
        if *game_tick < self.prep_ticks
            { return GameState::Prep; }

        // play
        if *game_tick < (self.prep_ticks + self.game_ticks)
            { return GameState::Play; }

        // game over
        return GameState::GameOver;
    }
}

//-------------------------------------------------------------------------------------------------------------------
