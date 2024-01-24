//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Game over report containing details summarizing a game.
///
/// This is an opaque type which contains the true game over report in serialized form.
#[serde_as]
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GameOverReport
{
    /// Data needed for a client to reassemble a game over report.
    #[serde_as(as = "Bytes")]
    pub serialized_game_over_data: Vec<u8>
}

impl GameOverReport
{
    pub fn new(report: Vec<u8>) -> GameOverReport
    {
        GameOverReport{ serialized_game_over_data: report }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Flag that contains the game over report once 'game over' occurs.
///
/// The game over report can only be taken once.
#[derive(Resource, Default, Debug)]
pub struct GameEndFlag
{
    set: bool,
    report: Option<GameOverReport>,
}

impl GameEndFlag
{
    /// Set the game over flag with a game over report.
    pub fn set(&mut self, report: GameOverReport)
    {
        self.set = true;
        self.report = Some(report);
    }

    /// Take the game over report if it exists.
    pub fn take_report(&mut self) -> Option<GameOverReport>
    {
        self.report.take()
    }

    /// Check if the flag is set.
    pub fn is_set(&self) -> bool
    {
        self.set
    }
}

//-------------------------------------------------------------------------------------------------------------------
