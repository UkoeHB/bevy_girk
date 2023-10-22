//local shortcuts
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Static information in a game app.
#[derive(Resource)]
pub struct ClickGameContext
{
    /// Seed for the game's deterministic random number generator.
    seed: u128,
    /// Game duration config.
    duration_config: GameDurationConfig,
}

impl ClickGameContext
{
    /// New game context
    pub fn new(
        seed            : u128,
        duration_config : GameDurationConfig,
    ) -> ClickGameContext 
    {
        ClickGameContext{ seed, duration_config }
    }

    pub fn seed(&self) -> u128                           { self.seed }
    pub fn duration_config(&self) -> &GameDurationConfig { &self.duration_config }
}

//-------------------------------------------------------------------------------------------------------------------
