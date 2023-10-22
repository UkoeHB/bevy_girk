//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Player report for the game over report.
#[derive(Serialize, Deserialize)]
pub struct ClickPlayerReport
{
    /// client id within the game
    pub client_id: ClientIdType,
    /// Player score during the game.
    pub score: PlayerScore,
}

//-------------------------------------------------------------------------------------------------------------------

/// Report emitted at the end of the game
#[derive(Serialize, Deserialize)]
pub struct ClickGameOverReport
{
    /// Total ticks that elapsed during the game.
    pub game_ticks: Ticks,

    /// Each player's individual report.
    pub player_reports: Vec<ClickPlayerReport>,
}

//-------------------------------------------------------------------------------------------------------------------
