//local shortcuts
use crate::click_game_integration::click_game::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy_replicon::prelude::ChannelKind;
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

impl IntoChannelKind for GameMsg
{
    fn into_event_type(&self) -> ChannelKind
    {
        match self
        {
            Self::RequestRejected{..} => SendUnordered.into(),
            Self::CurrentGameMode(_)  => SendOrdered.into(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
