//local shortcuts
use crate::*;

//third-party shortcuts
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
pub enum GameFWMsg
{
    /// The current game framework mode.
    CurrentGameFWMode(GameFWMode),
    /// Ping response to a ping request.
    PingResponse(PingResponse),
}

//-------------------------------------------------------------------------------------------------------------------
