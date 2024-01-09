//local shortcuts

//third-party shortcuts
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct PingRequest
{
    /// Timestamp of requester.
    pub timestamp_ns: u64
}

//-------------------------------------------------------------------------------------------------------------------

/// Requests that can be sent to the game framework.
//ClientFwRequest
//todo: impl Into<SendPolicy>
#[derive(Debug, Serialize, Deserialize)]
pub enum GameFwRequest
{
    /// Notify game framework of the client's initialization progress.
    //SetInitProgress
    ClientInitProgress(f32),
    /// Request a ping response.
    //GetPing
    PingRequest(PingRequest),
    /// Request the current game framework mode.
    //GetGameFwMode
    GameFwModeRequest,
}

//-------------------------------------------------------------------------------------------------------------------
