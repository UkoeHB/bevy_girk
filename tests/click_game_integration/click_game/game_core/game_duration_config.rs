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
    /// Number of ticks that should elapse in [GameMode::Prep] before switching [GameMode::Prep] -> [GameMode::Play].
    prep_ticks: Ticks,
    /// Number of ticks that should elapse in [GameMode::Play] before switching [GameMode::Play] -> [GameMode::GameOver].
    game_ticks: Ticks
    // The first 'game over' tick will occur after 'prep_ticks + game_ticks' ticks have elapsed.
}

impl GameDurationConfig
{
    pub fn new(
        prep_ticks : Ticks,
        game_ticks : Ticks
    ) -> GameDurationConfig
    {
        GameDurationConfig{
                prep_ticks,
                game_ticks
            }
    }

    pub fn expected_mode(&self, game_ticks_elapsed: Ticks) -> GameMode
    {
        // prep
        if game_ticks_elapsed.0 < self.prep_ticks.0
            { return GameMode::Prep; }

        // play
        if game_ticks_elapsed.0 < (self.prep_ticks.0 + self.game_ticks.0)
            { return GameMode::Play; }

        // game over
        return GameMode::GameOver;
    }
}

//-------------------------------------------------------------------------------------------------------------------
