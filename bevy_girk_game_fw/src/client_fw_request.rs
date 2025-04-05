//local shortcuts
use bevy_girk_utils::*;

//third-party shortcuts
use bevy_replicon::prelude::Channel;
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
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientFwRequest
{
    /// Notify game framework of the client's initialization progress.
    SetInitProgress(f32),
    /// Request a ping response.
    GetPing(PingRequest),
    /// Request the current game framework state.
    GetGameFwState,
}

impl IntoChannel for ClientFwRequest
{
    fn into_event_type(&self) -> Channel
    {
        match self
        {
            Self::SetInitProgress(_) => SendOrdered.into(),
            Self::GetPing(_)         => SendUnordered.into(),
            Self::GetGameFwState     => SendUnordered.into(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
