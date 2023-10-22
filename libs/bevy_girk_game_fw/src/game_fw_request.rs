//local shortcuts

//third-party shortcuts
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct PingRequest
{
    /// timestamp of requester
    pub timestamp_ns: u64
}

//-------------------------------------------------------------------------------------------------------------------

/// Requests that can be sent to the game framework.
#[derive(Debug, Serialize, Deserialize)]
pub enum GameFWRequest
{
    /// Notify game framework of the client's initialization progress.
    ClientInitProgress(f32),
    /// Request a ping response.
    PingRequest(PingRequest),
    /// Request the current game framework mode.
    GameFWModeRequest,
}

//-------------------------------------------------------------------------------------------------------------------
