//local shortcuts
use bevy_girk_utils::*;

//third-party shortcuts
use bevy_replicon::prelude::ChannelKind;
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Requests that can be sent to the game.
#[derive(Debug, Serialize, Deserialize)]
pub enum GameRequest
{
    GameStateRequest,
    ClickButton,
}

impl IntoChannelKind for GameRequest
{
    fn into_event_type(&self) -> ChannelKind
    {
        match self
        {
            Self::GameStateRequest => SendUnordered.into(),
            Self::ClickButton     => SendOrdered.into(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
