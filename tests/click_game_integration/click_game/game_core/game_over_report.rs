//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy_replicon::prelude::ClientId;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Player report for the game over report.
#[derive(Serialize, Deserialize)]
pub struct ClickPlayerReport
{
    /// client id within the game
    pub client_id: ClientId,
    /// Player score during the game.
    pub score: PlayerScore,
}

//-------------------------------------------------------------------------------------------------------------------

/// Report emitted at the end of the game
#[derive(Serialize, Deserialize)]
pub struct ClickGameOverReport
{
    /// The final game tick that elapsed.
    pub last_game_tick: Tick,

    /// Each player's individual report.
    pub player_reports: Vec<ClickPlayerReport>,
}

//-------------------------------------------------------------------------------------------------------------------
