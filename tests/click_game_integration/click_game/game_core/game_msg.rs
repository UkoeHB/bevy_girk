//local shortcuts
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Reasons a game request may be rejected
#[derive(Debug, Serialize, Deserialize)]
pub enum RejectionReason
{
    ModeMismatch,
    Invalid,
    None
}

//-------------------------------------------------------------------------------------------------------------------

/// Messages that can be sent out of the game.
#[derive(Debug, Serialize, Deserialize)]
pub enum GameMsg
{
    RequestRejected{ reason: RejectionReason, request: GameRequest },
    CurrentGameMode(GameMode),
}

//-------------------------------------------------------------------------------------------------------------------
