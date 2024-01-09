//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy_replicon::prelude::EventType;
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct PingResponse
{
    /// original ping request
    pub request: PingRequest,
    //note: the ping response will be wrapped in a message that contains the game ticks elapsed, so that does not need
    //      to be recorded here
}

//-------------------------------------------------------------------------------------------------------------------

/// Messages that can be sent out of the game framework.
#[derive(Debug, Serialize, Deserialize)]
pub enum GameFwMsg
{
    /// The current game framework mode.
    CurrentGameFwMode(GameFwMode),
    /// Ping response to a ping request.
    PingResponse(PingResponse),
}

impl IntoEventType for GameFwMsg
{
    fn into_event_type(&self) -> EventType
    {
        match self
        {
            Self::CurrentGameFwMode(_) => SendOrdered.into(),
            Self::PingResponse(_)      => SendUnordered.into(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
